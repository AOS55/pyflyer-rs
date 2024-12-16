use flyer::components::DubinsAircraftControls;
use numpy::{PyArray1, PyReadonlyArray1};
use pyo3::prelude::*;

pub mod dubins;
use dubins::ContinuousDubinsAct;

#[derive(Debug, Clone, Copy)]
pub enum ActionSpace {
    ContinuousDubinsAct(ContinuousDubinsAct),
}

pub trait ActionConverter {
    fn to_controls<'py>(
        &self,
        py: Python<'py>,
        action: PyReadonlyArray1<f64>,
    ) -> DubinsAircraftControls;
}

impl ActionSpace {
    pub fn new_dubins() -> Self {
        ActionSpace::ContinuousDubinsAct(ContinuousDubinsAct::new(0.0, 0.0, 0.0))
    }
}

impl ActionConverter for ActionSpace {
    fn to_controls<'py>(
        &self,
        py: Python<'py>,
        action: PyReadonlyArray1<'py, f64>,
    ) -> DubinsAircraftControls {
        match self {
            ActionSpace::ContinuousDubinsAct(converter) => converter.to_controls(py, action),
        }
    }
}
