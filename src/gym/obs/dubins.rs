use flyer::components::DubinsAircraftState;

pub struct ContinuousDubinsObs {
    pub heading: f64,
    pub altitude: f64,
    pub airspeed: f64,
}

impl ContinuousDubinsObs {
    pub fn new(heading: f64, altitude: f64, airspeed: f64) -> Self {
        Self {
            heading,
            altitude,
            airspeed,
        }
    }

    pub fn from_aircraft(aircraft_state: DubinsAircraftState) -> Self {
        let heading = aircraft_state.spatial.attitude.euler_angles().2;
        let altitude = aircraft_state.spatial.position[2];
        let airspeed = aircraft_state.spatial.velocity.norm();

        Self {
            heading,
            altitude,
            airspeed,
        }
    }
}
