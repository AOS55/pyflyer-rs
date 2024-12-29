use bevy::prelude::*;
use flyer::resources::{
    BiomeConfig, BiomeThresholds, FeatureConfig, HeightNoiseConfig, MoistureNoiseConfig,
    NoiseConfig, TerrainConfig,
};
use flyer::systems::terrain::noise::NoiseLayer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::gym::config::errors::ConfigError;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TerrainConfigBuilder {
    noise_builder: NoiseConfigBuilder,
    biome_builder: BiomeConfigBuilder,
    feature_builder: FeatureConfigBuilder,
    pub seed: u64,
}

impl TerrainConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn noise_config(mut self, builder: NoiseConfigBuilder) -> Self {
        self.noise_builder = builder;
        self
    }

    pub fn biome_config(mut self, builder: BiomeConfigBuilder) -> Self {
        self.biome_builder = builder;
        self
    }

    pub fn feature_config(mut self, builder: FeatureConfigBuilder) -> Self {
        self.feature_builder = builder;
        self
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Some(noise_config) = value.get("noise") {
            builder = builder.noise_config(NoiseConfigBuilder::from_json(noise_config)?);
        }

        if let Some(biome_config) = value.get("biome") {
            builder = builder.biome_config(BiomeConfigBuilder::from_json(biome_config)?);
        }

        if let Some(feature_config) = value.get("feature") {
            builder = builder.feature_config(FeatureConfigBuilder::from_json(feature_config)?);
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<TerrainConfig, ConfigError> {
        let config = TerrainConfig {
            seed: self.seed,
            noise: self.noise_builder.build()?,
            biome: self.biome_builder.build()?,
            feature: self.feature_builder.build()?,
            render: TerrainConfig::default().render,
        };

        Ok(config)
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct NoiseConfigBuilder {
    height: Option<HeightNoiseConfigBuilder>,
}

impl NoiseConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn height_noise(mut self, builder: HeightNoiseConfigBuilder) -> Self {
        self.height = Some(builder);
        self
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Some(height_config) = value.get("height") {
            builder = builder.height_noise(HeightNoiseConfigBuilder::from_json(height_config)?);
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<NoiseConfig, ConfigError> {
        Ok(NoiseConfig {
            height: self.height.unwrap_or_default().build()?,
            moisture: MoistureNoiseConfig::default(), // Don't edit moisture for now
            river: NoiseConfig::default().river,      // Don't edit rivers for now
        })
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct HeightNoiseConfigBuilder {
    scale: Option<f32>,
    octaves: Option<u32>,
    persistence: Option<f32>,
    lacunarity: Option<f32>,
    layers: Vec<NoiseLayer>,
}

impl HeightNoiseConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = Some(scale);
        self
    }

    pub fn octaves(mut self, octaves: u32) -> Self {
        self.octaves = Some(octaves);
        self
    }

    pub fn persistence(mut self, persistence: f32) -> Self {
        self.persistence = Some(persistence);
        self
    }

    pub fn lacunarity(mut self, lacunarity: f32) -> Self {
        self.lacunarity = Some(lacunarity);
        self
    }

    pub fn add_layer(mut self, layer: NoiseLayer) -> Self {
        self.layers.push(layer);
        self
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Some(scale) = value.get("scale").and_then(|v| v.as_f64()) {
            builder = builder.scale(scale as f32);
        }
        if let Some(octaves) = value.get("octaves").and_then(|v| v.as_u64()) {
            builder = builder.octaves(octaves as u32);
        }
        if let Some(persistence) = value.get("persistence").and_then(|v| v.as_f64()) {
            builder = builder.persistence(persistence as f32);
        }
        if let Some(lacunarity) = value.get("lacunarity").and_then(|v| v.as_f64()) {
            builder = builder.lacunarity(lacunarity as f32);
        }

        if let Some(layers) = value.get("layers").and_then(|v| v.as_array()) {
            for layer_value in layers {
                if let Some(noise_layer) = parse_noise_layer(layer_value)? {
                    builder = builder.add_layer(noise_layer);
                }
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<HeightNoiseConfig, ConfigError> {
        Ok(HeightNoiseConfig {
            scale: self.scale.unwrap_or(800.0),
            octaves: self.octaves.unwrap_or(4),
            persistence: self.persistence.unwrap_or(0.5),
            lacunarity: self.lacunarity.unwrap_or(2.0),
            layers: self.layers,
        })
    }
}

fn parse_noise_layer(value: &Value) -> Result<Option<NoiseLayer>, ConfigError> {
    let scale =
        value.get("scale").and_then(|v| v.as_f64()).ok_or_else(|| {
            ConfigError::ValidationError("scale is required for noise layer".into())
        })? as f32;

    let amplitude = value
        .get("amplitude")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| {
            ConfigError::ValidationError("amplitude is required for noise layer".into())
        })? as f32;

    let octaves = value
        .get("octaves")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ConfigError::ValidationError("octaves is required for noise layer".into()))?
        as u32;

    let mut layer = NoiseLayer::new(scale, amplitude, octaves);

    // Optional parameters
    if let Some(persistence) = value.get("persistence").and_then(|v| v.as_f64()) {
        layer = layer.with_persistence(persistence as f32);
    }

    if let Some(weight) = value.get("weight").and_then(|v| v.as_f64()) {
        layer = layer.with_weight(weight as f32);
    }

    if let Some(offset_x) = value.get("offset_x").and_then(|v| v.as_f64()) {
        if let Some(offset_y) = value.get("offset_y").and_then(|v| v.as_f64()) {
            layer = layer.with_offset(Vec2::new(offset_x as f32, offset_y as f32));
        }
    }

    Ok(Some(layer))
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct BiomeConfigBuilder {
    thresholds_builder: BiomeThresholdsBuilder,
}

impl BiomeConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn thresholds(mut self, builder: BiomeThresholdsBuilder) -> Self {
        self.thresholds_builder = builder;
        self
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Some(thresholds) = value.get("thresholds") {
            builder = builder.thresholds(BiomeThresholdsBuilder::from_json(thresholds)?);
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<BiomeConfig, ConfigError> {
        Ok(BiomeConfig {
            thresholds: self.thresholds_builder.build()?,
        })
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct BiomeThresholdsBuilder {
    water: Option<f32>,
    mountain_start: Option<f32>,
    mountain_width: Option<f32>,
    beach_width: Option<f32>,
    forest_moisture: Option<f32>,
    desert_moisture: Option<f32>,
    field_sizes: Option<[f32; 4]>,
}

impl BiomeThresholdsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Some(water) = value.get("water").and_then(|v| v.as_f64()) {
            builder.water = Some(water as f32);
        }
        if let Some(mountain_start) = value.get("mountain_start").and_then(|v| v.as_f64()) {
            builder.mountain_start = Some(mountain_start as f32);
        }
        if let Some(mountain_width) = value.get("mountain_width").and_then(|v| v.as_f64()) {
            builder.mountain_width = Some(mountain_width as f32);
        }
        if let Some(beach_width) = value.get("beach_width").and_then(|v| v.as_f64()) {
            builder.beach_width = Some(beach_width as f32);
        }
        if let Some(forest_moisture) = value.get("forest_moisture").and_then(|v| v.as_f64()) {
            builder.forest_moisture = Some(forest_moisture as f32);
        }
        if let Some(desert_moisture) = value.get("desert_moisture").and_then(|v| v.as_f64()) {
            builder.desert_moisture = Some(desert_moisture as f32);
        }

        if let Some(field_sizes) = value.get("field_sizes").and_then(|v| v.as_array()) {
            if field_sizes.len() == 4 {
                let mut sizes = Vec::with_capacity(4);
                for value in field_sizes {
                    if let Some(size) = value.as_f64() {
                        sizes.push(size as f32);
                    } else {
                        return Err(ConfigError::ValidationError(
                            "field_sizes values must be numbers".into(),
                        ));
                    }
                }
                builder.field_sizes = Some(sizes.try_into().unwrap()); // Safe because we checked len == 4
            } else {
                return Err(ConfigError::ValidationError(
                    "field_sizes must contain exactly 4 values".into(),
                ));
            }
        }

        builder.validate_thresholds()?;
        Ok(builder)
    }

    pub fn build(self) -> Result<BiomeThresholds, ConfigError> {
        Ok(BiomeThresholds {
            water: self.water.unwrap_or(0.48),
            mountain_start: self.mountain_start.unwrap_or(0.75),
            mountain_width: self.mountain_width.unwrap_or(0.1),
            beach_width: self.beach_width.unwrap_or(0.025),
            forest_moisture: self.forest_moisture.unwrap_or(0.95),
            desert_moisture: self.desert_moisture.unwrap_or(0.2),
            field_sizes: self.field_sizes.unwrap_or([96.0, 128.0, 256.0, 512.0]),
        })
    }

    pub fn validate_thresholds(&self) -> Result<(), ConfigError> {
        if let Some(water) = self.water {
            if !(0.0..=1.0).contains(&water) {
                return Err(ConfigError::ValidationError(
                    "water threshold must be between 0 and 1".into(),
                ));
            }
        }

        if let Some(mountain_start) = self.mountain_start {
            if !(0.0..=1.0).contains(&mountain_start) {
                return Err(ConfigError::ValidationError(
                    "mountain_start must be between 0 and 1".into(),
                ));
            }
        }

        if let (Some(mountain_start), Some(mountain_width)) =
            (self.mountain_start, self.mountain_width)
        {
            if mountain_start + mountain_width > 1.0 {
                return Err(ConfigError::ValidationError(
                    "mountain_start + mountain_width must not exceed 1.0".into(),
                ));
            }
        }

        if let (Some(forest_moisture), Some(desert_moisture)) =
            (self.forest_moisture, self.desert_moisture)
        {
            if forest_moisture <= desert_moisture {
                return Err(ConfigError::ValidationError(
                    "forest_moisture must be greater than desert_moisture".into(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct FeatureConfigBuilder;

impl FeatureConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(_value: &Value) -> Result<Self, ConfigError> {
        Ok(Self::new())
    }

    pub fn build(self) -> Result<FeatureConfig, ConfigError> {
        Ok(FeatureConfig::default())
    }
}
