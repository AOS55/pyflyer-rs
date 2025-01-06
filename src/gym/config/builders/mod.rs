use flyer::components::AircraftConfig;
use flyer::resources::{AgentConfig, UpdateMode};
use rand;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

mod aircraft;
// mod environment;
mod act;
mod obs;
mod physics;
mod reward;
mod start;
mod termination;
mod terrain;

use crate::gym::config::errors::ConfigError;
use crate::gym::{ActionSpace, EnvConfig, ObservationSpace};
use crate::utils::{RngManager, WithRng};
pub use aircraft::{
    create_aircraft_builder, AircraftBuilder, AircraftBuilderEnum, DubinsAircraftConfigBuilder,
    FullAircraftConfigBuilder,
};
// use environment::EnvironmentConfigBuilder;
pub use act::ActionSpaceBuilder;
pub use obs::ObservationSpaceBuilder;
pub use physics::PhysicsConfigBuilder;
use reward::RewardWeightsBuilder;
use start::RandomStartPosConfigBuilder;
use termination::TerminalConditionsBuilder;
pub use terrain::{HeightNoiseConfigBuilder, NoiseConfigBuilder, TerrainConfigBuilder};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvConfigBuilder {
    #[serde(skip)]
    pub rng_manager: Option<RngManager>,
    pub max_episode_steps: Option<u32>,
    pub steps_per_action: Option<usize>,
    pub time_step: Option<f64>,
    #[serde(skip)]
    pub aircraft_builders: HashMap<String, AircraftBuilderEnum>,
    #[serde(skip)]
    pub action_builders: HashMap<String, ActionSpaceBuilder>,
    #[serde(skip)]
    pub observation_builders: HashMap<String, ObservationSpaceBuilder>,
    #[serde(skip)]
    pub physics_builder: PhysicsConfigBuilder,
    // environment_builder: EnvironmentConfigBuilder,
    #[serde(skip)]
    pub terrain_builder: TerrainConfigBuilder,
    #[serde(skip)]
    pub reward_builder: RewardWeightsBuilder,
    #[serde(skip)]
    pub terminal_builder: TerminalConditionsBuilder,
}

impl Default for EnvConfigBuilder {
    fn default() -> Self {
        Self {
            rng_manager: None,
            max_episode_steps: None,
            steps_per_action: None,
            time_step: None,
            aircraft_builders: HashMap::new(),
            action_builders: HashMap::new(),
            observation_builders: HashMap::new(),
            physics_builder: PhysicsConfigBuilder::default(),
            // environment_builder: EnvironmentConfigBuilder::default(),
            terrain_builder: TerrainConfigBuilder::default(),
            reward_builder: RewardWeightsBuilder::default(),
            terminal_builder: TerminalConditionsBuilder::default(),
        }
    }
}

impl EnvConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_episode_steps(mut self, steps: u32) -> Self {
        self.max_episode_steps = Some(steps);
        self
    }

    pub fn steps_per_action(mut self, steps: usize) -> Self {
        self.steps_per_action = Some(steps);
        self
    }

    pub fn time_step(mut self, dt: f64) -> Self {
        self.time_step = Some(dt);
        self
    }

    pub fn terrain_config(mut self, builder: TerrainConfigBuilder) -> Self {
        self.terrain_builder = builder;
        self
    }

    pub fn from_json(json_value: &Value) -> Result<Self, ConfigError> {
        let mut builder = Self::new();

        // Parse seed
        let seed = json_value
            .get("seed")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(rand::random);

        let rng_manager = RngManager::new(seed);
        builder.rng_manager = Some(rng_manager.clone());

        // Parse basic configuration
        if let Some(steps) = json_value.get("max_episode_steps").and_then(|v| v.as_u64()) {
            builder = builder.max_episode_steps(steps as u32);
        }
        if let Some(steps) = json_value.get("steps_per_action").and_then(|v| v.as_u64()) {
            builder = builder.steps_per_action(steps as usize);
        }
        if let Some(dt) = json_value.get("time_step").and_then(|v| v.as_f64()) {
            builder = builder.time_step(dt);
        }

        // Parse aircraft configurations
        if let Some(aircraft_configs) = json_value.get("aircraft_config").and_then(|v| v.as_array())
        {
            for (i, config) in aircraft_configs.iter().enumerate() {
                let id = format!("aircraft_{}", i);
                let mut aircraft_agent = create_aircraft_builder(config)?;

                // Set id name in builder
                match &mut aircraft_agent.aircraft_builder {
                    AircraftBuilderEnum::Dubins(builder) => builder.name = Some(id.clone()),
                    AircraftBuilderEnum::Full(builder) => builder.name = Some(id.clone()),
                }

                // Initialize each aircraft with its own RNG stream
                builder.aircraft_builders.insert(
                    id.clone(),
                    aircraft_agent
                        .aircraft_builder
                        .with_rng(rng_manager.get_rng(&id)),
                );

                builder
                    .action_builders
                    .insert(id.clone(), aircraft_agent.action_builder);

                builder
                    .observation_builders
                    .insert(id.clone(), aircraft_agent.observation_builder);
            }
        }

        // Parse terrain configuration
        if let Some(terrain_config) = json_value.get("terrain_config") {
            let mut config = TerrainConfigBuilder::from_json(terrain_config)?;
            config.seed = seed;
            builder = builder.terrain_config(config);
        }

        if let Some(physics_config) = json_value.get("physics_config") {
            builder.physics_builder = PhysicsConfigBuilder::from_json(physics_config)?;
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<EnvConfig, ConfigError> {
        let rng_manager = self
            .rng_manager
            .unwrap_or_else(|| RngManager::new(rand::random()));

        // Build configurations from HashMaps
        let mut aircraft_configs: HashMap<String, AircraftConfig> = HashMap::new();
        let mut action_spaces: HashMap<String, ActionSpace> = HashMap::new();
        let mut observation_spaces: HashMap<String, ObservationSpace> = HashMap::new();

        // Process all builders
        for (id, builder) in self.aircraft_builders {
            aircraft_configs.insert(id.clone(), builder.build()?);
        }

        for (id, builder) in self.action_builders {
            action_spaces.insert(id.clone(), builder.build()?);
        }

        for (id, builder) in self.observation_builders {
            observation_spaces.insert(id.clone(), builder.build()?);
        }

        Ok(EnvConfig {
            seed: rng_manager.master_seed(),
            update_mode: UpdateMode::Gym,
            max_episode_steps: self.max_episode_steps.unwrap_or(1000),
            steps_per_action: self.steps_per_action.unwrap_or(10),
            time_step: self.time_step.unwrap_or(1.0 / 60.0),
            aircraft_configs,
            action_spaces,
            observation_spaces,
            physics_config: self.physics_builder.build()?,
            // environment_config: self.environment_builder.build()?,
            terrain_config: self.terrain_builder.build()?,
            agent_config: AgentConfig::default(),
            reward_weights: Some(self.reward_builder.build()?),
            terminal_conditions: self.terminal_builder.build()?,
        })
    }
}
