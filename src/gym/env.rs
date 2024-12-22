use bevy::prelude::*;
use flyer::{
    plugins::Id,
    resources::{AgentState, RenderMode},
};
use numpy::PyReadonlyArray1;
use pyo3::{prelude::*, types::PyDict};
use std::{
    collections::HashMap,
    env,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::gym::{act::ToControls, obs::FromAircraft, setup_app, EnvConfig};

#[pyclass(name = "FlyerEnv", unsendable)]
pub struct FlyerEnv {
    app: App,
    state: Arc<Mutex<AgentState>>,
    config: EnvConfig,
}

#[pymethods]
impl FlyerEnv {
    #[new]
    fn new(
        config_dict: Option<&Bound<'_, PyDict>>,
        render_mode: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Self> {
        // Parse configuration
        let mut config = if let Some(dict) = config_dict {
            EnvConfig::from_pydict(dict)?
        } else {
            EnvConfig::default()
        };

        // Set render_mode
        let render_mode = if let Some(render_py) = render_mode {
            if let Ok(render_str) = render_py.extract::<&str>() {
                RenderMode::from_str(render_str).map_err(|err| {
                    pyo3::exceptions::PyValueError::new_err(format!("Invalid render mode: {}", err))
                })?
            } else {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "Render mode must be a string",
                ));
            }
        } else {
            RenderMode::Human
        };

        config.agent_config.mode = render_mode;

        // Create Bevy app
        let mut app = App::new();

        // Get asset directorys in correct location
        let current_dir = env::current_dir().unwrap();
        let asset_path = current_dir
            .join("pyflyer-rs/flyer-rs/assets")
            .to_str()
            .unwrap()
            .to_string();

        app = setup_app(app, config.clone(), asset_path);

        // Create shared agent state
        let agent_state = AgentState::new(&config.agent_config);
        let agent_state_arc = Arc::new(Mutex::new(agent_state));
        let agent_state_clone = agent_state_arc.clone();

        app.run();

        Ok(Self {
            app,
            state: agent_state_clone,
            config,
        })
    }

    fn reset<'py>(
        &mut self,
        py: Python<'py>,
    ) -> PyResult<(Bound<'py, PyDict>, Bound<'py, PyDict>)> {
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
        action: &Bound<'py, PyDict>,
    ) -> PyResult<(Bound<'py, PyDict>, f64, bool, bool, Bound<'py, PyDict>)> {
        // Vaidate actions against configured aircraft
        for key in action.keys() {
            let id = key.extract::<String>()?;
            if !self.config.action_spaces.contains_key(&id) {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Received action for unconfigured aircraft {}",
                    id
                )));
            }
        }

        // Update action queue with validated actions
        self.update_action_queue(py, action)?;

        // Step simulation
        self.app.update();
        // for _ in 0..self.config.steps_per_action {
        //     // self.elapsed_time += self.config.time_step;
        //     self.app.update();
        // }

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
    fn get_observation<'py>(&mut self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let obs_dict = PyDict::new(py);

        if let Ok(state) = self.state.lock() {
            if let Ok(state_buffer) = state.state_buffer.lock() {
                // Process each aircraft state in the buffer
                for (id, aircraft_state) in state_buffer.iter() {
                    // Get the identifier string
                    let id_str = match id {
                        Id::Named(name) => name.clone(),
                        Id::Entity(entity) => format!("entity_{:?}", entity),
                    };

                    // Get the corresponding observation space configuration
                    if let Some(obs_space) = self.config.observation_spaces.get(&id_str) {
                        // Convert aircraft state to numpy array using FromAircraft trait
                        let np_array = obs_space.from_aircraft(py, aircraft_state)?;
                        obs_dict.set_item(id_str, np_array)?;
                    } else {
                        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                            "No observation space configured for aircraft {}",
                            id_str
                        )));
                    }
                }

                Ok(obs_dict)
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "Failed to acquire state buffer lock",
                ))
            }
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to acquire state lock",
            ))
        }
    }

    fn update_action_queue<'py>(
        &mut self,
        py: Python<'py>,
        action: &Bound<'py, PyDict>,
    ) -> PyResult<()> {
        let mut new_actions = HashMap::new();

        // Get a lock on state
        if let Ok(state) = self.state.lock() {
            for (key, value) in action.iter() {
                let id_str = key.extract::<String>()?;
                let id = Id::Named(id_str.clone());

                if let Some(action_space) = self.config.action_spaces.get(&id_str) {
                    let array: PyReadonlyArray1<f64> = value.extract()?;
                    let controls = action_space.to_controls(py, array);
                    new_actions.insert(id.into(), controls);
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                        "No action space configured for aircraft {}",
                        id_str
                    )));
                }
            }

            // Update the action queue
            if let Ok(mut action_queue) = state.action_queue.lock() {
                *action_queue = new_actions;
                Ok(())
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "Failed to acquire action queue lock",
                ))
            }
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Failed to acquire state lock",
            ))
        }
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
