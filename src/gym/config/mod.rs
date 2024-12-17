use flyer::{
    components::{AircraftConfig, TerminalConditions},
    resources::{EnvironmentConfig, PhysicsConfig, RewardWeights, TerrainConfig},
};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::{ActionSpace, ObservationSpace};

mod builders;
mod errors;
mod traits;

pub use builders::*;
pub use errors::*;
pub use traits::*;

#[derive(Debug, Clone)]
pub struct EnvConfig {
    // Master Seed
    pub seed: u64,

    // Time Configuration
    pub max_episode_steps: u32,
    pub steps_per_action: u32,
    pub time_step: f64,

    // Aircraft Configuration
    pub aircraft_configs: Vec<AircraftConfig>,
    pub physics_config: PhysicsConfig,
    // pub environment_config: EnvironmentConfig,

    // Terrain Configuration
    pub terrain_config: TerrainConfig,

    // Terminal conditions
    pub terminal_conditions: TerminalConditions,

    // Reward configuration
    pub reward_weights: Option<RewardWeights>,
}

impl EnvConfig {
    pub fn builder() -> EnvConfigBuilder {
        EnvConfigBuilder::new()
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let builder = EnvConfigBuilder::from_pydict(dict)?;
        builder.build().map_err(Into::into)
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        EnvConfigBuilder::new()
            .build()
            .expect("Default configuration should always be valid")
    }
}
