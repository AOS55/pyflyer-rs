use crate::gym::Space;

use flyer::components::{DubinsAircraftControls, DubinsAircraftState};
use numpy::PyArray1;
use pyo3::prelude::*;

pub trait ObservationBuilder {
    fn build_observation(&self, state: &DubinsAircraftState, py: Python) -> PyResult<PyObject>;
    fn get_observation_space(&self) -> Space;
}

pub trait ActionBuilder {
    fn build_controls(&self, action: &PyArray1<f64>) -> DubinsAircraftControls;
    fn get_action_space(&self) -> Space;
}

pub struct DubinsObservationBuilder;

impl ObservationBuilder for DubinsObservationBuilder {
    fn build_observation(&self, state: &DubinsAircraftState, py: Python) -> PyResult<PyObject> {
        let obs = vec![
            state.spatial.heading,
            state.spatial.position.x,
            state.spatial.position.y,
            state.spatial.position.z,
            state.spatial.velocity.norm(),
        ];

        Ok(PyArray1::from_vec(py, obs).into_py(py))
    }

    fn get_observation_space(&self) -> Space {
        Space::new(
            vec![5], // 5 dimensions: heading, pos_x, pos_y, pos_z, velocity
            vec![-std::f64::consts::PI, -10000.0, -10000.0, -10000.0, 0.0], // lower bounds
            vec![std::f64::consts::PI, 10000.0, 10000.0, 10000.0, 100.0], // upper bounds
        )
        .unwrap()
    }
}

pub struct DubinsActionBuilder {
    max_bank_angle: f64,
    max_acceleration: f64,
    max_vertical_speed: f64,
}

impl Default for DubinsActionBuilder {
    fn default() -> Self {
        Self {
            max_bank_angle: std::f64::consts::PI / 4.0, // 45 degrees
            max_acceleration: 5.0,                      // m/s^2
            max_vertical_speed: 10.0,                   // m/s
        }
    }
}

impl ActionBuilder for DubinsActionBuilder {
    fn build_controls(&self, action: &PyArray1<f64>) -> DubinsAircraftControls {
        let array = action.readonly();
        DubinsAircraftControls {
            bank_angle: array[0] * self.max_bank_angle,
            acceleration: array[1] * self.max_acceleration,
            vertical_speed: array[2] * self.max_vertical_speed,
        }
    }

    fn get_action_space(&self) -> Space {
        Space::new(
            vec![3],                // 3 dimensions: bank_angle, acceleration, vertical_speed
            vec![-1.0, -1.0, -1.0], // normalized bounds
            vec![1.0, 1.0, 1.0],
        )
        .unwrap()
    }
}
