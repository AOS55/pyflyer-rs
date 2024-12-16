use super::ActionConverter;
use flyer::components::DubinsAircraftControls;
use numpy::{PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct ContinuousDubinsAct {
    pub acceleration: f64,
    pub bank_angle: f64,
    pub vertical_speed: f64,
}

impl ContinuousDubinsAct {
    pub fn new(bank_angle: f64, acceleration: f64, vertical_speed: f64) -> Self {
        Self {
            bank_angle,
            acceleration,
            vertical_speed,
        }
    }

    /// Create an action to be sent to the Simulator loop
    fn create(&self) -> DubinsAircraftControls {
        DubinsAircraftControls {
            acceleration: self.acceleration,
            bank_angle: self.bank_angle,
            vertical_speed: self.vertical_speed,
        }
    }
}

impl ActionConverter for ContinuousDubinsAct {
    fn to_controls<'py>(
        &self,
        _py: Python<'py>,
        action: PyReadonlyArray1<f64>,
    ) -> DubinsAircraftControls {
        let array = action.as_array();
        DubinsAircraftControls {
            acceleration: array[0],
            bank_angle: array[1],
            vertical_speed: array[2],
        }
    }
}
