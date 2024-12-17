use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::config::ConfigError;
use crate::gym::ObservationSpace;

pub struct ObservationSpaceBuilder {
    obs_space: Option<ObservationSpace>,
}

impl Default for ObservationSpaceBuilder {
    fn default() -> Self {
        Self {
            obs_space: Some(ObservationSpace::ContinuousDubinsObs),
        }
    }
}

impl ObservationSpaceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn obs_space(mut self, obs_type: ObservationSpace) -> Self {
        self.obs_space = Some(obs_type);
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(obs_type) = dict.get_item("type")? {
            match obs_type.extract()? {
                "ContinuousDubinsObs" => {
                    builder = builder.obs_space(ObservationSpace::ContinuousDubinsObs);
                }
                _ => return Err(ConfigError::InvalidObservationType(obs_type.to_string()).into()),
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<ObservationSpace, ConfigError> {
        self.obs_space.ok_or(ConfigError::MissingObservationSpace)
    }
}
