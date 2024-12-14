use nalgebra::Vector3;
use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;

use flyer::components::AircraftState;

pub struct EnvState {
    aircraft_state: AircraftState,
    elapsed_time: f64,
}

impl EnvState {
    pub fn get_observation(&self, py: Python) -> PyResult<PyObject> {
        // Create a vector of observations
        // May need a Builder pattern (enum) here to get the observation based on env config

        let obs = vec![0.0f64; 10];

        Ok(PyArray1::from_vec(py, obs).into_py(py))
    }

    pub fn apply_action(&mut self, action: &PyArray1<f64>) {
        let controls = action;
        // Apply controls as before need a state builder pattern to apply the appropriate values
    }
}
