use flyer::resources::PhysicsConfig;
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::config::errors::ConfigError;

#[derive(Default)]
pub struct PhysicsConfigBuilder {
    pub max_velocity: Option<f64>,
    pub max_angular_velocity: Option<f64>,
    pub timestep: Option<f64>,
}

impl PhysicsConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_velocity(mut self, vel: f64) -> Self {
        self.max_velocity = Some(vel);
        self
    }

    pub fn max_angular_velocity(mut self, vel: f64) -> Self {
        self.max_angular_velocity = Some(vel);
        self
    }

    pub fn timestep(mut self, dt: f64) -> Self {
        self.timestep = Some(dt);
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(max_velocity) = dict.get_item("max_velocity")? {
            builder = builder.max_velocity(max_velocity.extract()?);
        }
        if let Some(max_angular_velocity) = dict.get_item("max_angular_velocity")? {
            builder = builder.max_angular_velocity(max_angular_velocity.extract()?);
        }
        if let Some(timestep) = dict.get_item("timestep")? {
            builder = builder.timestep(timestep.extract()?);
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<PhysicsConfig, ConfigError> {
        let mut config = PhysicsConfig::default();

        if let Some(max_velocity) = self.max_velocity {
            config.max_velocity = max_velocity;
        }
        if let Some(max_angular_velocity) = self.max_angular_velocity {
            config.max_angular_velocity = max_angular_velocity;
        }
        if let Some(timestep) = self.timestep {
            config.timestep = timestep;
        }

        Ok(config)
    }
}
