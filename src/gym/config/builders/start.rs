use flyer::components::RandomStartPosConfig;
use nalgebra::Vector2;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::utils::WithRng;

#[derive(Default, Debug, Clone)]
pub struct RandomStartPosConfigBuilder {
    origin: Option<Vector2<f64>>,
    variance: Option<f64>,
    min_altitude: Option<f64>,
    max_altitude: Option<f64>,
    rng: Option<ChaCha8Rng>,
}

impl RandomStartPosConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Ok(Some(config)) = dict.get_item("random_start") {
            if let Ok(config_dict) = config.downcast::<PyDict>() {
                let origin_x = config_dict
                    .get_item("origin_x")?
                    .and_then(|v| v.extract().ok());
                let origin_y = config_dict
                    .get_item("origin_y")?
                    .and_then(|v| v.extract().ok());

                if let (Some(x), Some(y)) = (origin_x, origin_y) {
                    builder.origin = Some(Vector2::new(x, y));
                }

                builder.variance = config_dict
                    .get_item("variance")?
                    .and_then(|v| v.extract().ok());
                builder.min_altitude = config_dict
                    .get_item("min_altitude")?
                    .and_then(|v| v.extract().ok());
                builder.max_altitude = config_dict
                    .get_item("max_altitude")?
                    .and_then(|v| v.extract().ok());
            }
        }

        Ok(builder)
    }

    pub fn build(&self) -> RandomStartPosConfig {
        let default_config = RandomStartPosConfig::default();

        RandomStartPosConfig {
            origin: self.origin.unwrap_or_else(|| default_config.origin),
            variance: self.variance.unwrap_or_else(|| default_config.variance),
            min_altitude: self
                .min_altitude
                .unwrap_or_else(|| default_config.min_altitude),
            max_altitude: self
                .max_altitude
                .unwrap_or_else(|| default_config.max_altitude),
            rng: self.rng.clone().unwrap_or_else(ChaCha8Rng::from_entropy),
        }
    }
}

impl WithRng for RandomStartPosConfigBuilder {
    fn with_rng(mut self, rng: ChaCha8Rng) -> Self {
        self.rng = Some(rng);
        self
    }
}
