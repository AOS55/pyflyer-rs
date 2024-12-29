use flyer::{
    components::{AircraftConfig, TerminalConditions},
    resources::{
        AgentConfig, EnvironmentConfig, PhysicsConfig, RewardWeights, TerrainConfig, UpdateMode,
    },
};
use serde_json::Value;
use std::collections::HashMap;

pub use crate::gym::{config::ConfigError, ActionSpace, ObservationSpace};

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
    pub steps_per_action: usize,
    pub time_step: f64,

    // Method to update
    pub update_mode: UpdateMode,

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
    pub fn from_json(json_str: &Value) -> Result<Self, ConfigError> {
        let builder = EnvConfigBuilder::from_json(json_str)?;
        builder.build()
    }
}

impl Default for EnvConfig {
    fn default() -> Self {
        EnvConfigBuilder::new()
            .build()
            .expect("Default configuration should always be valid")
    }
}
