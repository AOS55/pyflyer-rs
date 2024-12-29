use flyer::resources::RewardWeights;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};

use crate::gym::config::ConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardWeightsBuilder;

impl Default for RewardWeightsBuilder {
    fn default() -> Self {
        RewardWeightsBuilder
    }
}

impl RewardWeightsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_pydict(_dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let builder = Self::new();

        Ok(builder)
    }

    pub fn build(self) -> Result<RewardWeights, ConfigError> {
        let weights = RewardWeights::default();

        Ok(weights)
    }
}
