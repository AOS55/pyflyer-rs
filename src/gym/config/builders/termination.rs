use flyer::components::TerminalConditions;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};

use crate::gym::config::errors::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalConditionsBuilder;

impl Default for TerminalConditionsBuilder {
    fn default() -> Self {
        TerminalConditionsBuilder
    }
}

impl TerminalConditionsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pydict(_dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let builder = Self::new();

        Ok(builder)
    }

    pub fn build(self) -> Result<TerminalConditions, ConfigError> {
        let conditions = TerminalConditions::default();

        Ok(conditions)
    }
}
