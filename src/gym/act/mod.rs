use flyer::components::{AircraftControlSurfaces, AircraftControls, DubinsAircraftControls};
use serde::{Deserialize, Serialize};

pub trait ToControls {
    fn to_controls(&self, action: Vec<f64>) -> AircraftControls;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionSpace {
    Continuous(ContinuousActionSpace),
    Discrete(DiscreteActionSpace),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl ActionSpace {
    pub fn new_continuous_dubins() -> Self {
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

    pub fn new_continuous_full() -> Self {
        ActionSpace::Continuous(ContinuousActionSpace::FullAircraft {
            max_elevator: 10.0,
            max_aileron: 10.0,
            max_rudder: 10.0,
        })
    }

    pub fn new_discrete_full() -> Self {
        ActionSpace::Discrete(DiscreteActionSpace::FullAircraft {
            elevator_levels: vec![-1.0, 0.0, 1.0],
            aileron_levels: vec![-1.0, 0.0, 1.0],
            rudder_levels: vec![-1.0, 0.0, 1.0],
        })
    }
}

impl ToControls for ActionSpace {
    fn to_controls(&self, action: Vec<f64>) -> AircraftControls {
        match self {
            ActionSpace::Continuous(continuous) => continuous.to_controls(action),
            ActionSpace::Discrete(discrete) => discrete.to_controls(action),
        }
    }
}

impl ToControls for ContinuousActionSpace {
    fn to_controls<'py>(&self, action: Vec<f64>) -> AircraftControls {
        match self {
            ContinuousActionSpace::DubinsAircraft {
                max_acceleration,
                max_bank_angle,
                max_vertical_speed,
            } => AircraftControls::Dubins(DubinsAircraftControls {
                acceleration: action[0] * max_acceleration,
                bank_angle: action[1] * max_bank_angle,
                vertical_speed: action[2] * max_vertical_speed,
            }),
            ContinuousActionSpace::FullAircraft {
                max_elevator,
                max_aileron,
                max_rudder,
            } => AircraftControls::Full(AircraftControlSurfaces {
                elevator: action[0] * max_elevator,
                aileron: action[1] * max_aileron,
                rudder: action[2] * max_rudder,
                flaps: 0.0, // Not controlled in basic implementation
            }),
        }
    }
}

impl ToControls for DiscreteActionSpace {
    fn to_controls<'py>(&self, action: Vec<f64>) -> AircraftControls {
        let idx = action[0] as usize;

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
