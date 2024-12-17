mod builder;

pub use builder::ActionSpaceBuilder;

use flyer::components::{AircraftControlSurfaces, DubinsAircraftControls};
use numpy::PyReadonlyArray1;
use pyo3::prelude::*;

pub trait ToControls {
    fn to_controls<'py>(&self, py: Python<'py>, action: PyReadonlyArray1<f64>) -> AircraftControls;
}

#[derive(Debug, Clone)]
pub enum ActionSpace {
    Continuous(ContinuousActionSpace),
    Discrete(DiscreteActionSpace),
}

#[derive(Debug, Clone)]
pub enum ContinuousActionSpace {
    DubinsAircraft {
        // Maps normalized actions [-1, 1] to actual control ranges
        max_acceleration: f64,
        max_bank_angle: f64,
        max_vertical_speed: f64,
    },
    FullAircraft {
        max_elevator: f64,
        max_aileron: f64,
        max_rudder: f64,
    },
}

#[derive(Debug, Clone)]
pub enum DiscreteActionSpace {
    DubinsAircraft {
        acceleration_levels: Vec<f64>,
        bank_angle_levels: Vec<f64>,
        vertical_speed_levels: Vec<f64>,
    },
    FullAircraft {
        elevator_levels: Vec<f64>,
        aileron_levels: Vec<f64>,
        rudder_levels: Vec<f64>,
    },
}

#[derive(Debug, Clone)]
pub enum AircraftControls {
    Dubins(DubinsAircraftControls),
    Full(AircraftControlSurfaces),
}

impl ActionSpace {
    pub fn new_dubins() -> Self {
        ActionSpace::Continuous(ContinuousActionSpace::DubinsAircraft {
            max_acceleration: 10.0,
            max_bank_angle: 45.0_f64.to_radians(),
            max_vertical_speed: 5.0,
        })
    }

    pub fn new_discrete_dubins() -> Self {
        ActionSpace::Discrete(DiscreteActionSpace::DubinsAircraft {
            acceleration_levels: vec![-1.0, 0.0, 1.0],
            bank_angle_levels: vec![-0.5, 0.0, 0.5],
            vertical_speed_levels: vec![-1.0, 0.0, 1.0],
        })
    }
}

impl ToControls for ActionSpace {
    fn to_controls<'py>(&self, py: Python<'py>, action: PyReadonlyArray1<f64>) -> AircraftControls {
        match self {
            ActionSpace::Continuous(continuous) => continuous.to_controls(py, action),
            ActionSpace::Discrete(discrete) => discrete.to_controls(py, action),
        }
    }
}

impl ToControls for ContinuousActionSpace {
    fn to_controls<'py>(
        &self,
        _py: Python<'py>,
        action: PyReadonlyArray1<f64>,
    ) -> AircraftControls {
        let array = action.as_array();
        match self {
            ContinuousActionSpace::DubinsAircraft {
                max_acceleration,
                max_bank_angle,
                max_vertical_speed,
            } => AircraftControls::Dubins(DubinsAircraftControls {
                acceleration: array[0] * max_acceleration,
                bank_angle: array[1] * max_bank_angle,
                vertical_speed: array[2] * max_vertical_speed,
            }),
            ContinuousActionSpace::FullAircraft {
                max_elevator,
                max_aileron,
                max_rudder,
            } => AircraftControls::Full(AircraftControlSurfaces {
                elevator: array[0] * max_elevator,
                aileron: array[1] * max_aileron,
                rudder: array[2] * max_rudder,
                flaps: 0.0, // Not controlled in basic implementation
            }),
        }
    }
}

impl ToControls for DiscreteActionSpace {
    fn to_controls<'py>(
        &self,
        _py: Python<'py>,
        action: PyReadonlyArray1<f64>,
    ) -> AircraftControls {
        let array = action.as_array();
        let idx = array[0] as usize;

        match self {
            DiscreteActionSpace::DubinsAircraft {
                acceleration_levels,
                bank_angle_levels,
                vertical_speed_levels,
            } => AircraftControls::Dubins(DubinsAircraftControls {
                acceleration: acceleration_levels[idx % acceleration_levels.len()],
                bank_angle: bank_angle_levels[idx % bank_angle_levels.len()],
                vertical_speed: vertical_speed_levels[idx % vertical_speed_levels.len()],
            }),
            DiscreteActionSpace::FullAircraft {
                elevator_levels,
                aileron_levels,
                rudder_levels,
            } => AircraftControls::Full(AircraftControlSurfaces {
                elevator: elevator_levels[idx % elevator_levels.len()],
                aileron: aileron_levels[idx % aileron_levels.len()],
                rudder: rudder_levels[idx % rudder_levels.len()],
                flaps: 0.0, // Default flaps position
            }),
        }
    }
}
