use bevy::prelude::*;
use flyer::{
    components::{AircraftConfig, TerminalConditions},
    resources::{
        AgentConfig, EnvironmentConfig, PhysicsConfig, RewardWeights, TerrainConfig, UpdateMode,
    },
    utils::{RngManager, WithRng},
};
use serde_json::Value;
use std::collections::HashMap;

pub use crate::gym::{
    config::{
        builders::{
            DubinsAircraftConfigBuilder, FullAircraftConfigBuilder, HeightNoiseConfigBuilder,
            NoiseConfigBuilder, PhysicsConfigBuilder, TerrainConfigBuilder,
        },
        ConfigError,
    },
    ActionSpace, ObservationSpace,
};

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

impl EnvConfig {
    pub fn rebuild_with_seed(&self, new_seed: u64) -> Result<Self, ConfigError> {
        info!("Rebuilding config with seed: {}", new_seed);
        let mut builder = EnvConfigBuilder::new()
            .max_episode_steps(self.max_episode_steps)
            .steps_per_action(self.steps_per_action)
            .time_step(self.time_step);

        // Set up RNG manager with new seed
        let rng_manager = RngManager::new(new_seed);
        builder.rng_manager = Some(rng_manager.clone());

        // Rebuild aircraft configurations
        for (id, config) in &self.aircraft_configs {
            match config {
                AircraftConfig::Dubins(dubins) => {
                    let mut aircraft_builder = DubinsAircraftConfigBuilder::new();
                    aircraft_builder.name = Some(dubins.name.clone());
                    aircraft_builder.max_speed = Some(dubins.max_speed);
                    aircraft_builder.min_speed = Some(dubins.min_speed);
                    aircraft_builder.acceleration = Some(dubins.acceleration);
                    aircraft_builder.max_bank_angle = Some(dubins.max_bank_angle);
                    aircraft_builder.max_turn_rate = Some(dubins.max_turn_rate);
                    aircraft_builder.max_climb_rate = Some(dubins.max_climb_rate);
                    aircraft_builder.max_descent_rate = Some(dubins.max_descent_rate);

                    builder.aircraft_builders.insert(
                        id.clone(),
                        AircraftBuilderEnum::Dubins(
                            aircraft_builder.with_rng(rng_manager.get_rng(id)),
                        ),
                    );
                }
                AircraftConfig::Full(full) => {
                    let mut aircraft_builder = FullAircraftConfigBuilder::new();
                    aircraft_builder.name = Some(full.name.clone());
                    aircraft_builder.ac_type = Some(full.ac_type.clone());
                    aircraft_builder.mass = Some(full.mass.clone());
                    aircraft_builder.geometry = Some(full.geometry.clone());
                    aircraft_builder.aero_coef = Some(full.aero_coef.clone());

                    let builder_with_rng =
                        WithRng::with_rng(aircraft_builder, rng_manager.get_rng(id));
                    builder
                        .aircraft_builders
                        .insert(id.clone(), AircraftBuilderEnum::Full(builder_with_rng));
                }
            }

            // Preserve action and observation spaces
            builder.action_builders.insert(
                id.clone(),
                ActionSpaceBuilder::new().act_space(self.action_spaces.get(id).unwrap().clone()),
            );
            builder.observation_builders.insert(
                id.clone(),
                ObservationSpaceBuilder::new()
                    .obs_space(self.observation_spaces.get(id).unwrap().clone()),
            );
        }

        // Rebuild physics config
        builder.physics_builder = PhysicsConfigBuilder::new()
            .max_velocity(self.physics_config.max_velocity)
            .max_angular_velocity(self.physics_config.max_angular_velocity)
            .timestep(self.physics_config.timestep);

        // Rebuild terrain config with new seed
        let mut terrain_builder = TerrainConfigBuilder::new();
        terrain_builder.seed = new_seed;

        // Copy noise configuration
        let height_noise = HeightNoiseConfigBuilder::new()
            .scale(self.terrain_config.noise.height.scale)
            .octaves(self.terrain_config.noise.height.octaves)
            .persistence(self.terrain_config.noise.height.persistence)
            .lacunarity(self.terrain_config.noise.height.lacunarity);

        // Add all noise layers
        let mut final_height_builder = height_noise;
        for layer in &self.terrain_config.noise.height.layers {
            final_height_builder = final_height_builder.add_layer(layer.clone());
        }

        let noise_builder = NoiseConfigBuilder::new().height_noise(final_height_builder);

        builder = builder.terrain_config(terrain_builder.noise_config(noise_builder));

        // Build the final config
        builder.build()
    }
}
