use bevy::prelude::*;
use flyer::{components::RandomStartPosConfig, utils::WithRng};
use nalgebra::Vector2;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::gym::config::ConfigError;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct RandomStartPosConfigBuilder {
    origin: Option<Vector2<f64>>,
    variance: Option<f64>,
    min_altitude: Option<f64>,
    max_altitude: Option<f64>,
    seed: Option<u64>,
}

impl RandomStartPosConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_json(value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        if let Some(config) = value.get("random_start") {
            // Parse origin coordinates
            let origin_x = config.get("origin_x").and_then(|v| v.as_f64());
            let origin_y = config.get("origin_y").and_then(|v| v.as_f64());

            if let (Some(x), Some(y)) = (origin_x, origin_y) {
                builder.origin = Some(Vector2::new(x, y));
            }

            // Parse other parameters
            builder.variance = config.get("variance").and_then(|v| v.as_f64());
            builder.min_altitude = config.get("min_altitude").and_then(|v| v.as_f64());
            builder.max_altitude = config.get("max_altitude").and_then(|v| v.as_f64());
        }

        Ok(builder)
    }

    pub fn build(&self) -> RandomStartPosConfig {
        info!("Building RandomStartPosConfig with seed: {:?}", self.seed);

        let default_config = RandomStartPosConfig::default();

        let mut config = RandomStartPosConfig {
            origin: self.origin.unwrap_or_else(|| default_config.origin),
            variance: self.variance.unwrap_or_else(|| default_config.variance),
            min_altitude: self
                .min_altitude
                .unwrap_or_else(|| default_config.min_altitude),
            max_altitude: self
                .max_altitude
                .unwrap_or_else(|| default_config.max_altitude),
            seed: self.seed,
        };

        config.build()
    }
}

impl WithRng for RandomStartPosConfigBuilder {
    fn with_rng(mut self, rng: ChaCha8Rng) -> Self {
        let new_seed = rng.get_seed()[0] as u64;
        info!("Setting seed for RandomStartPosConfigBuilder: {}", new_seed);
        self.seed = Some(new_seed);
        self
    }
}
