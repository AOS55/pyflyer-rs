use flyer::{
    components::{AircraftConfig, TerminalConditions},
    resources::{AgentConfig, EnvironmentConfig, PhysicsConfig, RewardWeights, TerrainConfig},
};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use crate::gym::{ActionSpace, ObservationSpace};

mod builders;
mod errors;

pub use builders::*;
pub use errors::*;

#[derive(Debug, Clone)]
pub struct EnvConfig {
    // Master Seed
    pub seed: u64,

    // Time Configuration
    pub max_episode_steps: u32,
    pub steps_per_action: u32,
    pub time_step: f64,

    // Aircraft Configuration
    pub aircraft_configs: HashMap<String, AircraftConfig>,
    pub action_spaces: HashMap<String, ActionSpace>,
    pub observation_spaces: HashMap<String, ObservationSpace>,

    // Environment/physics Configuration
    pub physics_config: PhysicsConfig,
    // pub environment_config: EnvironmentConfig,

    // Terrain Configuration
    pub terrain_config: TerrainConfig,

    // Agent Configuration
    pub agent_config: AgentConfig,

    // Terminal conditions
    pub terminal_conditions: TerminalConditions,

    // Reward configuration
    pub reward_weights: Option<RewardWeights>,
}

impl EnvConfig {
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
