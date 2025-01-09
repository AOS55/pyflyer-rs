use flyer::components::{AircraftControlSurfaces, AircraftControls, DubinsAircraftControls};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait ToControls {
    fn to_controls(&self, action: HashMap<String, f64>) -> AircraftControls;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionSpace {
    Continuous(ContinuousActionSpace),
    Discrete(DiscreteActionSpace),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContinuousActionSpace {
    DubinsAircraft,
    FullAircraft,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DiscreteActionSpace {
    DubinsAircraft,
    FullAircraft,
}

impl ToControls for ActionSpace {
    fn to_controls(&self, action: HashMap<String, f64>) -> AircraftControls {
        match self {
            ActionSpace::Continuous(space) => space.to_controls(action),
            ActionSpace::Discrete(space) => space.to_controls(action),
        }
    }
}

/// These have the same implementation the I wanted to keep the enum to ensure the Id and enum are connected on the Python side
impl ToControls for ContinuousActionSpace {
    fn to_controls(&self, action: HashMap<String, f64>) -> AircraftControls {
        match self {
            ContinuousActionSpace::DubinsAircraft => {
                AircraftControls::Dubins(DubinsAircraftControls {
                    acceleration: action.get("acceleration").copied().unwrap_or(0.0),
                    bank_angle: action.get("bank_angle").copied().unwrap_or(0.0),
                    vertical_speed: action.get("vertical_speed").copied().unwrap_or(0.0),
                })
            }
            ContinuousActionSpace::FullAircraft => {
                AircraftControls::Full(AircraftControlSurfaces {
                    elevator: action.get("elevator").copied().unwrap_or(0.0),
                    aileron: action.get("aileron").copied().unwrap_or(0.0),
                    rudder: action.get("rudder").copied().unwrap_or(0.0),
                    flaps: 0.0,
                })
            }
        }
    }
}

impl ToControls for DiscreteActionSpace {
    fn to_controls(&self, action: HashMap<String, f64>) -> AircraftControls {
        match self {
            DiscreteActionSpace::DubinsAircraft => {
                AircraftControls::Dubins(DubinsAircraftControls {
                    acceleration: action.get("acceleration").copied().unwrap_or(0.0),
                    bank_angle: action.get("bank_angle").copied().unwrap_or(0.0),
                    vertical_speed: action.get("vertical_speed").copied().unwrap_or(0.0),
                })
            }
            DiscreteActionSpace::FullAircraft => AircraftControls::Full(AircraftControlSurfaces {
                elevator: action.get("elevator").copied().unwrap_or(0.0),
                aileron: action.get("aileron").copied().unwrap_or(0.0),
                rudder: action.get("rudder").copied().unwrap_or(0.0),
                flaps: 0.0,
            }),
        }
    }
}

impl ActionSpace {
    pub fn new_continuous_dubins() -> Self {
        ActionSpace::Continuous(ContinuousActionSpace::DubinsAircraft)
    }

    pub fn new_continuous_full() -> Self {
        ActionSpace::Continuous(ContinuousActionSpace::FullAircraft)
    }

    pub fn new_discrete_dubins() -> Self {
        ActionSpace::Discrete(DiscreteActionSpace::DubinsAircraft)
    }

    pub fn new_discrete_full() -> Self {
        ActionSpace::Discrete(DiscreteActionSpace::FullAircraft)
    }
}
