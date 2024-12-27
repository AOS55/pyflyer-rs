use flyer::components::AircraftConfig;
use flyer::resources::{AgentConfig, UpdateMode};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rand;
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
use aircraft::{create_aircraft_builder, AircraftBuilder, AircraftBuilderEnum};
// use environment::EnvironmentConfigBuilder;
pub use act::ActionSpaceBuilder;
pub use obs::ObservationSpaceBuilder;
use physics::PhysicsConfigBuilder;
use reward::RewardWeightsBuilder;
use start::RandomStartPosConfigBuilder;
use termination::TerminalConditionsBuilder;
use terrain::TerrainConfigBuilder;

pub struct EnvConfigBuilder {
    rng_manager: Option<RngManager>,
    max_episode_steps: Option<u32>,
    steps_per_action: Option<u32>,
    time_step: Option<f64>,
    aircraft_builders: HashMap<String, AircraftBuilderEnum>,
    action_builders: HashMap<String, ActionSpaceBuilder>,
    observation_builders: HashMap<String, ObservationSpaceBuilder>,
    physics_builder: PhysicsConfigBuilder,
    // environment_builder: EnvironmentConfigBuilder,
    terrain_builder: TerrainConfigBuilder,
    reward_builder: RewardWeightsBuilder,
    terminal_builder: TerminalConditionsBuilder,
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

    pub fn steps_per_action(mut self, steps: u32) -> Self {
        self.steps_per_action = Some(steps);
        self
    }

    pub fn time_step(mut self, dt: f64) -> Self {
        self.time_step = Some(dt);
        self
    }

    // pub fn physics_config(mut self, builder: PhysicsConfigBuilder) -> Self {
    //     self.physics_builder = builder;
    //     self
    // }

    pub fn terrain_config(mut self, builder: TerrainConfigBuilder) -> Self {
        self.terrain_builder = builder;
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        // Use the seed if passed, otherwise generate a random new one
        let seed = if let Some(seed) = dict.get_item("seed")? {
            seed.extract()?
        } else {
            rand::random()
        };
        let rng_manager = RngManager::new(seed);
        builder.rng_manager = Some(rng_manager.clone());

        // Set the max episode steps, steps per action, and time step if passed
        if let Some(steps) = dict.get_item("max_episode_steps")? {
            builder = builder.max_episode_steps(steps.extract()?);
        }
        if let Some(steps) = dict.get_item("steps_per_action")? {
            builder = builder.steps_per_action(steps.extract()?);
        }
        if let Some(dt) = dict.get_item("time_step")? {
            builder = builder.time_step(dt.extract()?);
        }

        // Get the aircraft from the list of aircraft types
        if let Some(aircraft_list) = dict.get_item("aircraft_config")? {
            if let Ok(aircraft_configs) = aircraft_list.downcast::<PyList>() {
                for (i, aircraft_dict) in aircraft_configs.iter().enumerate() {
                    if let Ok(config_dict) = aircraft_dict.downcast::<PyDict>() {
                        let aircraft_agent = create_aircraft_builder(&config_dict)?;
                        let id = format!("aircraft_{}", i);

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
            }
        }

        // Get the terrain configuration
        if let Some(terrain_dict) = dict.get_item("terrain_config")? {
            if let Ok(dict) = terrain_dict.downcast::<PyDict>() {
                let mut config = TerrainConfigBuilder::from_pydict(&dict)?;
                config.seed = seed;
                builder = builder.terrain_config(config);
            }
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
