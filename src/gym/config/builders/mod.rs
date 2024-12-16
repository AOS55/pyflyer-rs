use flyer::components::AircraftConfig;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use rand;

mod act;
mod aircraft;
mod environment;
mod obs;
mod physics;
mod reward;
mod start;
mod termination;
mod terrain;

use crate::gym::config::errors::ConfigError;
use crate::gym::config::EnvConfig;
use crate::utils::{RngManager, WithRng};
use act::ActionSpaceBuilder;
use aircraft::{create_aircraft_builder, AircraftBuilder, AircraftBuilderEnum};
use environment::EnvironmentConfigBuilder;
use obs::ObservationSpaceBuilder;
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
    observation_space: ObservationSpaceBuilder,
    action_space: ActionSpaceBuilder,
    aircraft_builders: Vec<AircraftBuilderEnum>,
    physics_builder: PhysicsConfigBuilder,
    environment_builder: EnvironmentConfigBuilder,
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
            observation_space: ObservationSpaceBuilder::default(),
            action_space: ActionSpaceBuilder::default(),
            aircraft_builders: Vec::new(),
            physics_builder: PhysicsConfigBuilder::default(),
            environment_builder: EnvironmentConfigBuilder::default(),
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

    pub fn seed(mut self, seed: u64) -> Self {
        self.rng_manager = Some(RngManager::new(seed));
        self
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

    pub fn aircraft_config(mut self, builder: AircraftBuilderEnum) -> Self {
        self.aircraft_builders.push(builder);
        self
    }

    pub fn physics_config(mut self, builder: PhysicsConfigBuilder) -> Self {
        self.physics_builder = builder;
        self
    }

    pub fn terrain_config(mut self, builder: TerrainConfigBuilder) -> Self {
        self.terrain_builder = builder;
        self
    }

    pub fn observation_space(mut self, builder: ObservationSpaceBuilder) -> Self {
        self.observation_space = builder;
        self
    }

    pub fn action_space(mut self, builder: ActionSpaceBuilder) -> Self {
        self.action_space = builder;
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

        // Get the aircrft from the list of aircraft types
        if let Some(aircraft_list) = dict.get_item("aircraft_config")? {
            if let Ok(aircraft_configs) = aircraft_list.downcast::<PyList>() {
                for (i, aircraft_dict) in aircraft_configs.iter().enumerate() {
                    if let Ok(config_dict) = aircraft_dict.downcast::<PyDict>() {
                        let aircraft_builder = create_aircraft_builder(&config_dict)?
                            .with_rng(rng_manager.get_rng(&format!("aircraft_{}", i)));
                        builder.aircraft_builders.push(aircraft_builder);
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

        // Build the Observation and Action Spaces
        if let Some(observation_dict) = dict.get_item("observation_config")? {
            if let Ok(dict) = observation_dict.downcast::<PyDict>() {
                let obs_space = ObservationSpaceBuilder::from_pydict(&dict)?;
                builder = builder.observation_space(obs_space);
            }
        }

        if let Some(action_dict) = dict.get_item("action_config")? {
            if let Ok(dict) = action_dict.downcast::<PyDict>() {
                let act_space = ActionSpaceBuilder::from_pydict(&dict)?;
                builder = builder.action_space(act_space);
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<EnvConfig, ConfigError> {
        let rng_manager = self
            .rng_manager
            .unwrap_or_else(|| RngManager::new(rand::random()));

        let aircraft_configs = self
            .aircraft_builders
            .into_iter()
            .map(|builder| builder.build())
            .collect::<Result<Vec<AircraftConfig>, ConfigError>>()?;

        Ok(EnvConfig {
            seed: rng_manager.master_seed(),
            max_episode_steps: self.max_episode_steps.unwrap_or(1000),
            steps_per_action: self.steps_per_action.unwrap_or(4),
            time_step: self.time_step.unwrap_or(1.0 / 60.0),
            observation_space: self.observation_space.build()?,
            action_space: self.action_space.build()?,
            aircraft_configs,
            physics_config: self.physics_builder.build()?,
            environment_config: self.environment_builder.build()?,
            terrain_config: self.terrain_builder.build()?,
            reward_weights: Some(self.reward_builder.build()?),
            terminal_conditions: self.terminal_builder.build()?,
        })
    }
}
