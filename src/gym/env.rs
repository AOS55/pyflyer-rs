use bevy::prelude::*;
use flyer::plugins::{
    add_aircraft_plugin, DubinsAircraftPlugin, FullAircraftPlugin, TerrainPlugin,
};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::{EnvConfig, EnvState};

#[pyclass(name = "FlyerEnv", unsendable)]
pub struct FlyerEnv {
    app: App,
    state: EnvState,
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
            app,
            state: EnvState::new(),
            config,
        })
    }

    fn reset<'py>(&mut self) -> PyResult<(Bound<'_, PyAny>, Bound<'_, PyAny>)> {
        // Reset the simulation/game
        todo!("Implement reset method")
    }

    fn step<'py>(
        &mut self,
        py: Python<'_>,
        action: &Bound<'py, PyAny>,
    ) -> PyResult<(Bound<'py, PyAny>, f64, bool, bool, Bound<'py, PyAny>)> {
        // Take a step in the simulation/game
        todo!("Implement step method")
    }
}
