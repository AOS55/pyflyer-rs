use bevy::prelude::*;
use flyer::components::{DubinsAircraftState, PlayerController};
use flyer::plugins::{
    add_aircraft_plugin, AgentPlugin, CameraPlugin, TerrainPlugin, TransformationPlugin,
};
use flyer::resources::AgentState;
use numpy::{PyArray1, PyArrayMethods};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::sync::{Arc, Mutex};

use crate::gym::EnvConfig;

#[pyclass(name = "FlyerEnv", unsendable)]
pub struct FlyerEnv {
    app: App,
    state: Arc<Mutex<AgentState>>,
    config: EnvConfig,
}

#[pymethods]
impl FlyerEnv {
    #[new]
    fn new(config_dict: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        // Parse configuration
        let config = if let Some(dict) = config_dict {
            EnvConfig::from_pydict(dict)?
        } else {
            EnvConfig::default()
        };

        // Create Bevy app
        let mut app = App::new();

        // Add minimal base plugin
        app.add_plugins(MinimalPlugins);

        // Create shared agent state
        let agent_state = AgentState::new(&config.agent_config);
        let agent_state_arc = Arc::new(Mutex::new(agent_state));
        let agent_state_clone = agent_state_arc.clone();

        // Create core interaction plugins (including terrain)
        app.add_plugins((
            TransformationPlugin::new(1.0),
            TerrainPlugin::with_config(config.terrain_config.clone()),
        ));

        // Add plugin for each aircraft configuration (ususally just one)
        for aircraft_config in config.aircraft_configs.iter() {
            add_aircraft_plugin(&mut app, aircraft_config.1.clone());
        }

        // Load camera plugin
        app.add_plugins(CameraPlugin);

        // Create agent plugin
        app.add_plugins(AgentPlugin::new(config.agent_config.clone()));

        Ok(Self {
            app,
            state: agent_state_clone,
            config,
        })
    }

    fn reset<'py>(&mut self, py: Python<'py>) -> PyResult<(Bound<'py, PyAny>, Bound<'py, PyDict>)> {
        if let Ok(mut state) = self.state.lock() {
            state.episode_count += 1;
            state.current_step = 0;
            state.terminated = false;
            state.truncated = false;
        }

        self.app.update();

        let initial_obs = self.get_observation(py)?;
        let initial_info = self.build_info_dict(py);

        Ok((initial_obs, initial_info))
    }

    fn step<'py>(
        &mut self,
        py: Python<'py>,
        action: &Bound<'py, PyAny>,
    ) -> PyResult<(Bound<'py, PyAny>, f64, bool, bool, Bound<'py, PyDict>)> {
        // Convert python Action to aircraft controls
        let action_array = action.downcast::<PyArray1<f64>>()?;
        let action_readonly = action_array.readonly();

        // let controls = match self.action_space.to_controls(py, action_readonly) {
        //     AircraftControls::Dubins(controls) => controls,
        //     AircraftControls::Full(_) => {
        //         return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
        //             "Full aircraft controls not supported yet",
        //         ))
        //     }
        // };

        // TODO: Move this to apply_action plugin system
        let world = &mut self.app.world_mut();
        let mut query = world.query_filtered::<&mut DubinsAircraftState, With<PlayerController>>();
        // if let Ok(mut aircraft_state) = query.get_single_mut(world) {
        //     aircraft_state.controls = controls;
        // };

        // Step simulation
        for _ in 0..self.config.steps_per_action {
            // self.elapsed_time += self.config.time_step;
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

    #[getter]
    fn get_action_space<'py>(&self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        // Get the action space
        todo!("Implement get_action_space method")
    }

    #[getter]
    fn get_observation_space<'py>(&self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        // Get the observation space
        todo!("Implement get_observation_space method")
    }

    #[getter]
    fn get_spec<'py>(&self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        // Get the environment spec
        todo!("Implement get_spec attribute")
    }

    #[getter]
    fn get_render_mode(&self) -> PyResult<String> {
        // Get the render mode
        todo!("Implement get_render_mode attribute")
    }

    #[getter]
    fn get_seed(&self) -> PyResult<u64> {
        // Get the seed
        Ok(self.config.seed)
    }

    #[getter]
    fn get_observation<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        // Get the current observation
        // let world = &mut self.app.world_mut();
        // let mut query = world.query_filtered::<&DubinsAircraftState, With<PlayerController>>();

        // if let Ok(aircraft_state) = query.get_single(world) {
        //     // Convert to observation based on the observation space type
        //     match self.observation_space {
        //         ObservationSpace::ContinuousDubinsObs => {
        //             // Create observation from aircraft state
        //             let obs = ContinuousDubinsObs::from_aircraft(*aircraft_state);

        //             // Convert to numpy array
        //             let observation = vec![
        //                 obs.heading,  // Heading angle in radians
        //                 obs.altitude, // Altitude in meters
        //                 obs.airspeed, // Airspeed in m/s
        //             ];

        //             // Create and return numpy array
        //             let np_array = PyArray1::from_vec(py, observation);
        //             Ok(np_array.into_any())
        //         }
        //     }
        // } else {
        //     // Return error if we couldn't get the aircraft state
        //     Err(ConfigError::ValidationError("Could not get aircraft state".into()).into())
        // }

        todo!("Implement get_observation method")
    }

    fn build_info_dict<'py>(&self, py: Python<'py>) -> Bound<'py, PyDict> {
        // Build info dictionary
        // TODO correctly implement logic
        PyDict::new(py)
    }

    fn calculate_reward(&mut self) -> f64 {
        // Calculate reward based on current state\
        0.0
    }

    fn check_termination(&mut self) -> (bool, bool) {
        // Check episode termination
        todo!("Implement termination check");
        // let terminated = false;

        // // Check truncation
        // let truncated = self.state.steps_count >= self.config.max_episode_steps;

        // return (terminated, truncated);
    }
}
