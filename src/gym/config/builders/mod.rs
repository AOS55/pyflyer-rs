use flyer::components::PhysicsModel;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand;

mod actions;
mod aircraft;
mod environment;
mod observations;
mod physics;
mod reward;
mod termination;
mod terrain;

use crate::gym::config::errors::ConfigError;
use crate::gym::config::EnvConfig;
use crate::utils::{RngManager, WithRng};
use aircraft::AircraftConfigBuilder;
use environment::EnvironmentConfigBuilder;
use physics::PhysicsConfigBuilder;
use reward::RewardWeightsBuilder;
use termination::TerminalConditionsBuilder;
use terrain::TerrainConfigBuilder;

#[derive(Default)]
pub struct EnvConfigBuilder {
    rng_manager: Option<RngManager>,
    max_episode_steps: Option<u32>,
    steps_per_action: Option<u32>,
    time_step: Option<f64>,
    aircraft_builder: AircraftConfigBuilder,
    physics_builder: PhysicsConfigBuilder,
    environment_builder: EnvironmentConfigBuilder,
    terrain_builder: TerrainConfigBuilder,
    reward_builder: RewardWeightsBuilder,
    terminal_builder: TerminalConditionsBuilder,
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

    pub fn aircraft_config(mut self, builder: AircraftConfigBuilder) -> Self {
        self.aircraft_builder = builder;
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

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        // Use the seed if passed, otherwise generate a random new one
        let seed = if let Some(seed) = dict.get_item("seed")? {
            seed.extract()?
        } else {
            rand::random()
        };
        let rng_manager = RngManager::new(seed);
        builder.rng_manager = Some(rng_manager);

        if let Some(steps) = dict.get_item("max_episode_steps")? {
            builder = builder.max_episode_steps(steps.extract()?);
        }
        if let Some(steps) = dict.get_item("steps_per_action")? {
            builder = builder.steps_per_action(steps.extract()?);
        }
        if let Some(dt) = dict.get_item("time_step")? {
            builder = builder.time_step(dt.extract()?);
        }

        let rng_manager = RngManager::new(seed);

        if let Some(aircraft_dict) = dict.get_item("aircraft_config")? {
            if let Ok(dict) = aircraft_dict.downcast::<PyDict>() {
                let mut config = AircraftConfigBuilder::from_pydict(&dict)?;
                config = config.with_rng(rng_manager.get_rng("aircraft"));
                builder = builder.aircraft_config(config);
            }
        }

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

        Ok(EnvConfig {
            seed: rng_manager.master_seed(),
            max_episode_steps: self.max_episode_steps.unwrap_or(1000),
            steps_per_action: self.steps_per_action.unwrap_or(4),
            time_step: self.time_step.unwrap_or(1.0 / 60.0),
            aircraft_config: self.aircraft_builder.build()?,
            physics_model: self.physics_builder.model.unwrap_or(PhysicsModel::Simple),
            physics_config: self.physics_builder.build()?,
            environment_config: self.environment_builder.build()?,
            terrain_config: self.terrain_builder.build()?,
            reward_weights: Some(self.reward_builder.build()?),
            terminal_conditions: self.terminal_builder.build()?,
        })
    }
}
