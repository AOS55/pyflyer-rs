use flyer::resources::{AtmosphereConfig, AtmosphereType, EnvironmentConfig, WindConfig};
use nalgebra::Vector3;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::config::errors::ConfigError;

#[derive(Default)]
pub struct EnvironmentConfigBuilder {
    wind_builder: Option<WindConfigBuilder>,
    atmosphere_builder: Option<AtmosphereConfigBuilder>,
    seed: Option<u64>,
}

impl EnvironmentConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seed(mut self, master_seed: u64) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        master_seed.hash(&mut hasher);
        "environment".hash(&mut hasher);
        self.seed = Some(hasher.finish());
        self
    }

    pub fn wind_config(mut self, builder: WindConfigBuilder) -> Self {
        self.wind_builder = Some(builder);
        self
    }

    pub fn atmosphere_config(mut self, builder: AtmosphereConfigBuilder) -> Self {
        self.atmosphere_builder = Some(builder);
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(wind_dict) = dict.get_item("wind_model_config")? {
            if let Ok(dict) = wind_dict.downcast::<PyDict>() {
                builder = builder.wind_config(WindConfigBuilder::from_pydict(&dict)?);
            }
        }

        if let Some(atmosphere_dict) = dict.get_item("atmosphere_config")? {
            if let Ok(dict) = atmosphere_dict.downcast::<PyDict>() {
                builder = builder.atmosphere_config(AtmosphereConfigBuilder::from_pydict(&dict)?);
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<EnvironmentConfig, ConfigError> {
        let mut wind_builder = self.wind_builder.unwrap_or_default();
        let mut atm_builder = self.atmosphere_builder.unwrap_or_default();

        if let Some(seed) = self.seed {
            wind_builder = wind_builder.with_seed(seed);
            atm_builder = atm_builder.with_seed(seed);
        }

        Ok(EnvironmentConfig {
            wind_model_config: wind_builder.build()?,
            atmosphere_config: atm_builder.build()?,
        })
    }
}

#[derive(Default)]
pub struct WindConfigBuilder {
    wind_type: Option<String>,
    velocity: Option<Vector3<f64>>,
    // Logarithmic model parameters
    d: Option<f64>,
    z0: Option<f64>,
    u_star: Option<f64>,
    bearing: Option<f64>,
    // Power law parameters
    u_r: Option<f64>,
    z_r: Option<f64>,
    alpha: Option<f64>,

    // Generation seed
    seed: Option<u64>,
}

impl WindConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seed(mut self, master_seed: u64) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        master_seed.hash(&mut hasher);
        "wind".hash(&mut hasher);
        self.seed = Some(hasher.finish());
        self
    }

    pub fn wind_type(mut self, type_str: &str) -> Self {
        self.wind_type = Some(type_str.to_string());
        self
    }

    pub fn velocity(mut self, velocity: Vector3<f64>) -> Self {
        self.velocity = Some(velocity);
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        let wind_type = dict
            .get_item("type")?
            .ok_or_else(|| ConfigError::MissingRequired("wind_type".into()))?
            .extract::<String>()?;

        builder = builder.wind_type(&wind_type);

        match wind_type.as_str() {
            "Constant" => {
                if let Some(velocity) = dict.get_item("velocity")? {
                    let (x, y, z): (f64, f64, f64) = velocity.extract()?;
                    builder = builder.velocity(Vector3::new(x, y, z));
                }
            }
            "Logarithmic" => {
                if let Some(d) = dict.get_item("d")? {
                    builder.d = Some(d.extract()?);
                }
                if let Some(z0) = dict.get_item("z0")? {
                    builder.z0 = Some(z0.extract()?);
                }
                if let Some(u_star) = dict.get_item("u_star")? {
                    builder.u_star = Some(u_star.extract()?);
                }
                if let Some(bearing) = dict.get_item("bearing")? {
                    builder.bearing = Some(bearing.extract()?);
                }
            }
            "PowerLaw" => {
                if let Some(u_r) = dict.get_item("u_r")? {
                    builder.u_r = Some(u_r.extract()?);
                }
                if let Some(z_r) = dict.get_item("z_r")? {
                    builder.z_r = Some(z_r.extract()?);
                }
                if let Some(alpha) = dict.get_item("alpha")? {
                    builder.alpha = Some(alpha.extract()?);
                }
                if let Some(bearing) = dict.get_item("bearing")? {
                    builder.bearing = Some(bearing.extract()?);
                }
            }
            _ => {
                return Err(ConfigError::InvalidParameter {
                    name: "wind_type".into(),
                    value: wind_type,
                }
                .into())
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<WindConfig, ConfigError> {
        match self.wind_type.as_deref() {
            Some("Constant") => {
                let velocity = self.velocity.ok_or_else(|| {
                    ConfigError::MissingRequired("velocity for Constant wind".into())
                })?;
                Ok(WindConfig::Constant { velocity })
            }
            Some("Logarithmic") => Ok(WindConfig::Logarithmic {
                d: self.d.unwrap_or(0.0),
                z0: self.z0.unwrap_or(0.0),
                u_star: self.u_star.unwrap_or(0.0),
                bearing: self.bearing.unwrap_or(0.0),
            }),
            Some("PowerLaw") => Ok(WindConfig::PowerLaw {
                u_r: self.u_r.unwrap_or(0.0),
                z_r: self.z_r.unwrap_or(0.0),
                alpha: self.alpha.unwrap_or(0.0),
                bearing: self.bearing.unwrap_or(0.0),
            }),
            _ => Err(ConfigError::InvalidParameter {
                name: "wind_type".into(),
                value: self.wind_type.unwrap_or_default(),
            }),
        }
    }
}

#[derive(Default)]
pub struct AtmosphereConfigBuilder {
    model_type: Option<AtmosphereType>,
    sea_level_density: Option<f64>,
    sea_level_temperature: Option<f64>,
    seed: Option<u64>,
}

impl AtmosphereConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_seed(mut self, master_seed: u64) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        master_seed.hash(&mut hasher);
        "atmosphere".hash(&mut hasher);
        self.seed = Some(hasher.finish());
        self
    }

    pub fn model_type(mut self, model_type: AtmosphereType) -> Self {
        self.model_type = Some(model_type);
        self
    }

    pub fn sea_level_density(mut self, density: f64) -> Self {
        self.sea_level_density = Some(density);
        self
    }

    pub fn sea_level_temperature(mut self, temperature: f64) -> Self {
        self.sea_level_temperature = Some(temperature);
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(model_type) = dict.get_item("model_type")? {
            builder = builder.model_type(match model_type.extract::<String>()?.as_str() {
                "Constant" => AtmosphereType::Constant,
                _ => AtmosphereType::Standard,
            });
        }

        if let Some(density) = dict.get_item("sea_level_density")? {
            builder = builder.sea_level_density(density.extract()?);
        }

        if let Some(temperature) = dict.get_item("sea_level_temperature")? {
            builder = builder.sea_level_temperature(temperature.extract()?);
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<AtmosphereConfig, ConfigError> {
        Ok(AtmosphereConfig {
            model_type: self.model_type.unwrap_or(AtmosphereType::Standard),
            sea_level_density: self.sea_level_density.unwrap_or(1.225),
            sea_level_temperature: self.sea_level_temperature.unwrap_or(288.15),
        })
    }
}
