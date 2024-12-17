use bevy::prelude::*;
use flyer::components::{DubinsAircraftState, PlayerController};
use flyer::plugins::{
    add_aircraft_plugin, DubinsAircraftPlugin, FullAircraftPlugin, TerrainPlugin,
};
use numpy::{IntoPyArray, PyArray1, PyArrayMethods, PyReadonlyArray1};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::act::ActionConverter;
use crate::gym::config::ConfigError;
use crate::gym::obs::dubins::ContinuousDubinsObs;
use crate::gym::{EnvConfig, ObservationSpace};

#[pyclass(name = "FlyerEnv", unsendable)]
pub struct FlyerEnv {
    // Time Tracking
    elapsed_time: f64,
    steps_count: u32,
    episode_count: u32,

    // Bevy reference
    app: App,

    // Current state tracking
    current_observation: Option<Vec<f64>>,
    last_action: Option<Vec<f64>>,

    config: EnvConfig,
}

#[pymethods]
impl FlyerEnv {
    #[new]
    fn new(config_dict: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        // Build the Bevy app (setup the simulation/game)
        let config = if let Some(dict) = config_dict {
            EnvConfig::from_pydict(dict)?
        } else {
            EnvConfig::default()
        };

        let mut app = App::new();

        // Add base plugin
        app.add_plugins(MinimalPlugins);

        // Add plugin for each aircraft configuration
        for aircraft_config in config.aircraft_configs.iter() {
            add_aircraft_plugin(&mut app, aircraft_config.clone());
        }

        app.add_plugins(TerrainPlugin::with_config(config.terrain_config.clone()));

        Ok(Self {
            elapsed_time: 0.0,
            steps_count: 0,
            episode_count: 0,
            app,
            current_observation: None,
            last_action: None,
            config,
        })
    }

    fn reset<'py>(&mut self) -> PyResult<(Bound<'_, PyAny>, Bound<'_, PyAny>)> {
        // Reset the simulation/game
        todo!("Implement reset method")
    }

    fn step<'py>(
        &mut self,
        py: Python<'py>,
        action: &Bound<'py, PyAny>,
    ) -> PyResult<(Bound<'py, PyAny>, f64, bool, bool, Bound<'py, PyDict>)> {
        // Convert python Action to aircraft controls
        let action_array = action.downcast::<PyArray1<f64>>()?;
        let action_readonly = action_array.readonly();
        let controls = self.config.action_space.to_controls(py, action_readonly);

        let world = &mut self.app.world_mut();
        let mut query = world.query_filtered::<&mut DubinsAircraftState, With<PlayerController>>();
        if let Ok(mut aircraft_state) = query.get_single_mut(world) {
            aircraft_state.controls = controls;
        };

        // Step simulation
        for _ in 0..self.config.steps_per_action {
            self.app.update();
        }

        // Calculate reward
        let reward = self.calculate_reward();

        // Check termination/truncation
        let (terminated, truncated) = self.check_termination();

        // Build observation and info
        let observation = self.get_observation(py)?;
        let info = self.build_info_dict(py);

        Ok((observation, reward, terminated, truncated, info))
    }

    fn render<'py>(&mut self, py: Python<'_>, mode: Option<String>) -> PyResult<Bound<'py, PyAny>> {
        // Render the simulation/game
        todo!("Implement render method")
    }

    fn get_action_space<'py>(&self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        // Get the action space
        todo!("Implement get_action_space method")
    }

    fn get_observation_space<'py>(&self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        // Get the observation space
        todo!("Implement get_observation_space method")
    }

    fn get_spec<'py>(&self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        // Get the environment spec
        todo!("Implement get_spec attribute")
    }

    fn get_render_mode(&self) -> PyResult<String> {
        // Get the render mode
        todo!("Implement get_render_mode attribute")
    }

    fn get_seed(&self) -> PyResult<u64> {
        // Get the seed
        Ok(self.config.seed)
    }

    fn get_observation<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // Get the current observation
        let world = &mut self.app.world_mut();
        let mut query = world.query_filtered::<&DubinsAircraftState, With<PlayerController>>();

        if let Ok(aircraft_state) = query.get_single(world) {
            // Convert to observation based on the observation space type
            match self.config.observation_space {
                ObservationSpace::ContinuousDubinsObs => {
                    // Create observation from aircraft state
                    let obs = ContinuousDubinsObs::from_aircraft(*aircraft_state);

                    // Convert to numpy array
                    let observation = vec![
                        obs.heading,  // Heading angle in radians
                        obs.altitude, // Altitude in meters
                        obs.airspeed, // Airspeed in m/s
                    ];

                    // Create and return numpy array
                    let np_array = PyArray1::from_vec(py, observation);
                    Ok(np_array.into_any())
                }
            }
        } else {
            // Return error if we couldn't get the aircraft state
            Err(ConfigError::ValidationError("Could not get aircraft state".into()).into())
        }
    }

    fn build_info_dict<'py>(&self, py: Python<'py>) -> Bound<'py, PyDict> {
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
        let terminated = false;

        // Check truncation
        let truncated = self.steps_count >= self.config.max_episode_steps;

        return (terminated, truncated);
    }
}
