use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::config::errors::ConfigError;

pub struct ObservationsConfigBuilder;

impl Default for ObservationsConfigBuilder {
    fn default() -> Self {
        ObservationsConfigBuilder
    }
}

impl ObservationsConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let builder = Self::new();

        if let Some()

        Ok(builder)
    }

    pub fn build(self) -> Result<(), ConfigError> {
        Ok(())
    }
}
