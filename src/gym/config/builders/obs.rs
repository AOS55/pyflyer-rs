use crate::gym::config::ConfigError;
use crate::gym::obs::ContinuousObservationSpace;
use crate::gym::ObservationSpace;

pub struct ObservationSpaceBuilder {
    obs_space: Option<ObservationSpace>,
}

impl Default for ObservationSpaceBuilder {
    fn default() -> Self {
        Self {
            obs_space: Some(ObservationSpace::Continuous(
                ContinuousObservationSpace::DubinsAircraft,
            )),
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

    pub fn build(self) -> Result<ObservationSpace, ConfigError> {
        self.obs_space.ok_or(ConfigError::MissingObservationSpace)
    }
}
