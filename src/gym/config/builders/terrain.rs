use bevy::prelude::*;
use flyer::resources::{
    BiomeConfig, BiomeThresholds, FeatureConfig, HeightNoiseConfig, MoistureNoiseConfig,
    NoiseConfig, TerrainConfig,
};
use flyer::systems::terrain::noise::NoiseLayer;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::gym::config::errors::ConfigError;

#[derive(Default)]
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

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(noise_dict) = dict.get_item("noise")? {
            if let Ok(dict) = noise_dict.downcast::<PyDict>() {
                builder = builder.noise_config(NoiseConfigBuilder::from_pydict(&dict)?);
            }
        }

        if let Some(biome_dict) = dict.get_item("biome")? {
            if let Ok(dict) = biome_dict.downcast::<PyDict>() {
                builder = builder.biome_config(BiomeConfigBuilder::from_pydict(&dict)?);
            }
        }

        if let Some(feature_dict) = dict.get_item("feature")? {
            if let Ok(dict) = feature_dict.downcast::<PyDict>() {
                builder = builder.feature_config(FeatureConfigBuilder::from_pydict(&dict)?);
            }
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

#[derive(Default)]
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

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(height_dict) = dict.get_item("height")? {
            if let Ok(dict) = height_dict.downcast::<PyDict>() {
                builder = builder.height_noise(HeightNoiseConfigBuilder::from_pydict(&dict)?);
            }
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

#[derive(Default)]
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

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(scale) = dict.get_item("scale")? {
            builder = builder.scale(scale.extract()?);
        }
        if let Some(octaves) = dict.get_item("octaves")? {
            builder = builder.octaves(octaves.extract()?);
        }
        if let Some(persistence) = dict.get_item("persistence")? {
            builder = builder.persistence(persistence.extract()?);
        }
        if let Some(lacunarity) = dict.get_item("lacunarity")? {
            builder = builder.lacunarity(lacunarity.extract()?);
        }

        fn parse_noise_layer(dict: &Bound<'_, PyDict>) -> PyResult<NoiseLayer> {
            let scale: f32 = dict
                .get_item("scale")?
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "scale is required for noise layer",
                    )
                })?
                .extract()?;

            let amplitude: f32 = dict
                .get_item("amplitude")?
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "amplitude is required for noise layer",
                    )
                })?
                .extract()?;

            let octaves: u32 = dict
                .get_item("octaves")?
                .ok_or_else(|| {
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "octaves is required for noise layer",
                    )
                })?
                .extract()?;

            let mut layer = NoiseLayer::new(scale, amplitude, octaves);

            // Optional parameters
            if let Some(persistence) = dict.get_item("persistence")? {
                layer = layer.with_persistence(persistence.extract()?);
            }

            if let Some(weight) = dict.get_item("weight")? {
                layer = layer.with_weight(weight.extract()?);
            }

            if let Some(offset_x) = dict.get_item("offset_x")? {
                if let Some(offset_y) = dict.get_item("offset_y")? {
                    layer = layer.with_offset(Vec2::new(offset_x.extract()?, offset_y.extract()?));
                }
            }

            Ok(layer)
        }

        if let Some(layers_item) = dict.get_item("layers")? {
            if let Ok(layers_list) = layers_item.downcast::<PyList>() {
                for layer in layers_list {
                    if let Ok(layer_dict) = layer.downcast::<PyDict>() {
                        if let Ok(noise_layer) = parse_noise_layer(&layer_dict) {
                            builder = builder.add_layer(noise_layer);
                        }
                    }
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

#[derive(Default)]
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

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(thresholds_dict) = dict.get_item("thresholds")? {
            if let Ok(dict) = thresholds_dict.downcast::<PyDict>() {
                builder = builder.thresholds(BiomeThresholdsBuilder::from_pydict(&dict)?);
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<BiomeConfig, ConfigError> {
        Ok(BiomeConfig {
            thresholds: self.thresholds_builder.build()?,
        })
    }
}

#[derive(Default)]
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

    pub fn water(mut self, threshold: f32) -> Self {
        self.water = Some(threshold);
        self
    }

    pub fn mountain_start(mut self, threshold: f32) -> Self {
        self.mountain_start = Some(threshold);
        self
    }

    pub fn mountain_width(mut self, width: f32) -> Self {
        self.mountain_width = Some(width);
        self
    }

    pub fn beach_width(mut self, width: f32) -> Self {
        self.beach_width = Some(width);
        self
    }

    pub fn forest_moisture(mut self, threshold: f32) -> Self {
        self.forest_moisture = Some(threshold);
        self
    }

    pub fn desert_moisture(mut self, threshold: f32) -> Self {
        self.desert_moisture = Some(threshold);
        self
    }

    pub fn field_sizes(mut self, sizes: [f32; 4]) -> Self {
        self.field_sizes = Some(sizes);
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(water) = dict.get_item("water")? {
            builder = builder.water(water.extract()?);
        }

        if let Some(mountain_start) = dict.get_item("mountain_start")? {
            builder = builder.mountain_start(mountain_start.extract()?);
        }

        if let Some(mountain_width) = dict.get_item("mountain_width")? {
            builder = builder.mountain_width(mountain_width.extract()?);
        }

        if let Some(beach_width) = dict.get_item("beach_width")? {
            builder = builder.beach_width(beach_width.extract()?);
        }

        if let Some(forest_moisture) = dict.get_item("forest_moisture")? {
            builder = builder.forest_moisture(forest_moisture.extract()?);
        }

        if let Some(desert_moisture) = dict.get_item("desert_moisture")? {
            builder = builder.desert_moisture(desert_moisture.extract()?);
        }

        // Handle field sizes array
        if let Some(field_sizes_py) = dict.get_item("field_sizes")? {
            if let Ok(sizes) = field_sizes_py.extract::<Vec<f32>>() {
                if sizes.len() == 4 {
                    let array: [f32; 4] = sizes.try_into().map_err(|_| {
                        ConfigError::ValidationError(
                            "field_sizes must contain exactly 4 values".into(),
                        )
                    })?;
                    builder = builder.field_sizes(array);
                } else {
                    return Err(ConfigError::ValidationError(
                        "field_sizes must contain exactly 4 values".into(),
                    )
                    .into());
                }
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<BiomeThresholds, ConfigError> {
        // Validate thresholds
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

        // TODO: Change to use the defaults from the aircraft if possible
        // Create BiomeThresholds with defaults for unspecified values
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
        // Additional validation logic can be added here
        // For example, checking relationships between thresholds
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

#[derive(Default)]
pub struct FeatureConfigBuilder;

impl FeatureConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pydict(_dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        Ok(Self::new())
    }

    pub fn build(self) -> Result<FeatureConfig, ConfigError> {
        Ok(FeatureConfig::default())
    }
}
