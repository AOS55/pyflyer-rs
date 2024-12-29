use flyer::components::AircraftState;
use serde::{Deserialize, Serialize};

pub trait ToObservation {
    fn to_observation(&self, state: &AircraftState) -> Vec<f64>;
}

#[derive(Copy, Debug, Clone, Serialize, Deserialize)]
pub enum ObservationSpace {
    Continuous(ContinuousObservationSpace),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContinuousObservationSpace {
    DubinsAircraft,
    FullAircraft,
}

impl Default for ObservationSpace {
    fn default() -> Self {
        ObservationSpace::Continuous(ContinuousObservationSpace::DubinsAircraft)
    }
}

impl ToObservation for ObservationSpace {
    fn to_observation(&self, state: &AircraftState) -> Vec<f64> {
        match self {
            ObservationSpace::Continuous(continuous) => continuous.to_observation(state),
        }
    }
}

impl ToObservation for ContinuousObservationSpace {
    fn to_observation(&self, state: &AircraftState) -> Vec<f64> {
        match (self, state) {
            (ContinuousObservationSpace::DubinsAircraft, AircraftState::Dubins(dubins_state)) => {
                // Convert DubinsAircraftState to simplified observation vector
                let mut obs = Vec::with_capacity(3);

                // Get heading from attitude quaternion (yaw angle)
                let euler = dubins_state.spatial.attitude.euler_angles();
                let heading = euler.2; // yaw angle

                // Altitude is negative of the z-component in NED frame
                let altitude = -dubins_state.spatial.position.z;

                // Airspeed from velocity magnitude
                let airspeed = dubins_state.spatial.velocity.magnitude();

                obs.extend_from_slice(&[heading, altitude, airspeed]);

                // Convert to numpy array
                obs
            }
            (ContinuousObservationSpace::FullAircraft, AircraftState::Full(full_state)) => {
                // Convert FullAircraftState to simplified observation vector for RL
                let mut obs = Vec::with_capacity(9);

                // Attitude (roll, pitch, yaw)
                let euler = full_state.spatial.attitude.euler_angles();
                obs.extend_from_slice(&[euler.0, euler.1, euler.2]); // roll, pitch, yaw

                // Angular rates (p, q, r)
                obs.extend_from_slice(&full_state.spatial.angular_velocity.as_slice());

                // Key flight parameters
                obs.push(full_state.air_data.true_airspeed); // airspeed
                obs.push(full_state.air_data.alpha); // angle of attack
                obs.push(full_state.air_data.beta); // sideslip angle

                // Convert to numpy array
                obs
            }
            _ => Vec::new(), // Return empty vector for mismatched types
        }
    }
}
