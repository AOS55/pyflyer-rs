use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::config::errors::ConfigError;

pub struct ActionsConfigBuilder;

impl Default for ActionsConfigBuilder {
    fn default() -> Self {
        ActionsConfigBuilder
    }
}

impl ActionsConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pydict(_dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let builder = Self::new();

        Ok(builder)
    }

    pub fn build(self) -> Result<(), ConfigError> {
        Ok(())
    }
}
