use bevy::prelude::*;
use flyer::components::{
    RandomHeadingConfig, RandomPosConfig, RandomSpeedConfig, RandomStartConfig,
};
use nalgebra::Vector2;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::gym::config::ConfigError;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RandomStartConfigBuilder {
    // Position configuration
    pub position: RandomPosConfigBuilder,
    // Speed configuration
    pub speed: RandomSpeedConfigBuilder,
    // Heading configuration
    pub heading: RandomHeadingConfigBuilder,
    // Common seed for random number generation
    pub seed: Option<u64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RandomPosConfigBuilder {
    pub origin: Option<Vector2<f64>>,
    pub variance: Option<f64>,
    pub min_altitude: Option<f64>,
    pub max_altitude: Option<f64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RandomSpeedConfigBuilder {
    pub min_speed: Option<f64>,
    pub max_speed: Option<f64>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RandomHeadingConfigBuilder {
    pub min_heading: Option<f64>,
    pub max_heading: Option<f64>,
}

impl RandomStartConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(value: &Value, seed: u64) -> Result<Self, ConfigError> {
        let mut builder = Self::new();
        builder.seed = Some(seed);

        if let Some(config) = value.get("random_start") {
            // Parse position configuration
            if let Some(pos_config) = config.get("position") {
                builder.position = RandomPosConfigBuilder::from_json(pos_config)?;
            }

            // Parse speed configuration
            if let Some(speed_config) = config.get("speed") {
                builder.speed = RandomSpeedConfigBuilder::from_json(speed_config)?;
            }

            // Parse heading configuration
            if let Some(heading_config) = config.get("heading") {
                builder.heading = RandomHeadingConfigBuilder::from_json(heading_config)?;
            }
        }

        Ok(builder)
    }

    pub fn build(&self) -> RandomStartConfig {
        info!("Building RandomStartConfig with seed: {:?}", self.seed);

        RandomStartConfig {
            position: self.position.build(),
            speed: self.speed.build(),
            heading: self.heading.build(),
            seed: self.seed,
        }
    }

    pub fn build_with_seed(&self, seed: u64) -> RandomStartConfig {
        RandomStartConfig {
            position: self.position.build(),
            speed: self.speed.build(),
            heading: self.heading.build(),
            seed: Some(seed),
        }
    }
}

impl RandomPosConfigBuilder {
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::default();

        // Parse origin coordinates
        let origin_x = value.get("origin_x").and_then(|v| v.as_f64());
        let origin_y = value.get("origin_y").and_then(|v| v.as_f64());

        if let (Some(x), Some(y)) = (origin_x, origin_y) {
            builder.origin = Some(Vector2::new(x, y));
        }

        // Parse other parameters
        builder.variance = value.get("variance").and_then(|v| v.as_f64());
        builder.min_altitude = value.get("min_altitude").and_then(|v| v.as_f64());
        builder.max_altitude = value.get("max_altitude").and_then(|v| v.as_f64());

        Ok(builder)
    }

    pub fn build(&self) -> RandomPosConfig {
        let default_config = RandomPosConfig::default();

        RandomPosConfig {
            origin: self.origin.unwrap_or_else(|| default_config.origin),
            variance: self.variance.unwrap_or_else(|| default_config.variance),
            min_altitude: self
                .min_altitude
                .unwrap_or_else(|| default_config.min_altitude),
            max_altitude: self
                .max_altitude
                .unwrap_or_else(|| default_config.max_altitude),
        }
    }
}

impl RandomSpeedConfigBuilder {
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::default();

        builder.min_speed = value.get("min_speed").and_then(|v| v.as_f64());
        builder.max_speed = value.get("max_speed").and_then(|v| v.as_f64());

        Ok(builder)
    }

    pub fn build(&self) -> RandomSpeedConfig {
        let default_config = RandomSpeedConfig::default();

        RandomSpeedConfig {
            min_speed: self.min_speed.unwrap_or_else(|| default_config.min_speed),
            max_speed: self.max_speed.unwrap_or_else(|| default_config.max_speed),
        }
    }
}

impl RandomHeadingConfigBuilder {
    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::default();

        builder.min_heading = value.get("min_heading").and_then(|v| v.as_f64());
        builder.max_heading = value.get("max_heading").and_then(|v| v.as_f64());

        Ok(builder)
    }

    pub fn build(&self) -> RandomHeadingConfig {
        let default_config = RandomHeadingConfig::default();

        RandomHeadingConfig {
            min_heading: self
                .min_heading
                .unwrap_or_else(|| default_config.min_heading),
            max_heading: self
                .max_heading
                .unwrap_or_else(|| default_config.max_heading),
        }
    }
}
