use bevy::prelude::*;
use flyer::plugins::{AircraftPlugin, TerrainPlugin};
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

        // let physics_model = if let Some(dict) = config_dict {
        //     if let Ok(Some(value)) = dict.get_item("physics_model") {
        //         if let Ok(model_str) = value.extract::<&str>() {
        //             if model_str == "full" {
        //                 PhysicsModel::Full
        //             } else {
        //                 PhysicsModel::Simple
        //             }
        //         } else {
        //             PhysicsModel::Simple
        //         }
        //     } else {
        //         PhysicsModel::Simple
        //     }
        // } else {
        //     PhysicsModel::Simple
        // };

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AircraftPlugin::new(config.physics_model),
            TerrainPlugin,
        ));

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
