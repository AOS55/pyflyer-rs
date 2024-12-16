use bevy::prelude::*;
use flyer::components::{DubinsAircraftState, PlayerController};
use flyer::systems::dubins_gym_control_system;
use numpy::{PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::Arc;

use crate::gym::EnvConfig;

pub struct EnvState {
    // Time Tracking
    elapsed_time: f64,
    steps_count: u32,
    episode_count: u32,

    // Bevy reference
    app: App,

    // Current state tracking
    current_observation: Option<Vec<f64>>,
    last_action: Option<Vec<f64>>,

    // Configurtation reference
    config: EnvConfig,
}

impl EnvState {
    pub fn new(app: App, config: EnvConfig) -> Self {
        Self {
            elapsed_time: 0.0,
            steps_count: 0,
            episode_count: 0,

            app,

            current_observation: None,
            last_action: None,

            config,
        }
    }

    pub fn step<'py>(
        &mut self,
        py: Python,
        action: &PyReadonlyArray1<'py, f64>,
    ) -> PyResult<(
        Bound<'py, PyArray1<f64>>, // observation
        f64,                       // reward
        bool,                      // terminated
        bool,                      // truncated
        Bound<'py, PyDict>,        // info
    )> {
        // Apply action to world
        self.config.act_space.to_controls(action);

        // Step simulation
        for _ in 0..self.config.steps_per_action {
            self.app.update();
        }

        // Calculate reward
        let reward = self.calculate_reward();

        // Check termination/truncation
        let (terminated, truncated) = self.check_termination();

        // Build observation and info
        let observation = self.get_observation()?;
        let info = self.build_info_dict()?;

        Ok((observation, reward, terminated, truncated, info))
    }

    pub fn get_observation<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<f64>>> {
        // Create a vector (or individual) observation

        Ok(PyArray1::from_vec(py, obs).into_py(py))
    }

    pub fn apply_action(&mut self, action: &PyArray1<f64>) -> Result<(), String> {
        let controls = self.config.act_space.apply_action(action);
        let world = &mut self.app.world_mut();

        // Run the control system
        let mut query = world.query_filtered::<&mut DubinsAircraftState, With<PlayerController>>();
        if let Ok(mut aircraft_state) = query.get_single_mut(world) {
            aircraft_state.controls = controls;
        };

        Ok(())
    }

    fn build_info_dict(&self) -> Result<PyObject, String> {
        // Build info dictionary
        todo!("Implement info dictionary")
    }

    fn calculate_reward(&mut self) -> f64 {
        // Calculate reward based on current state
        todo!("Implement reward calculation");
        0.0
    }

    fn check_termination(&mut self) -> (bool, bool) {
        // Check episode termination
        todo!("Implement termination check");
        let terminated = False;

        // Check truncation
        let truncated = self.steps_count >= self.config.max_steps;

        return (terminated, truncated);
    }
}
