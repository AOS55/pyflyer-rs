use bevy::prelude::*;
use flyer::components::AircraftState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait ToObservation {
    fn to_observation(&self, state: &AircraftState) -> HashMap<String, f64>;
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
    fn to_observation(&self, state: &AircraftState) -> HashMap<String, f64> {
        match self {
            ObservationSpace::Continuous(continuous) => continuous.to_observation(state),
        }
    }
}

impl ToObservation for ContinuousObservationSpace {
    fn to_observation(&self, state: &AircraftState) -> HashMap<String, f64> {
        match (self, state) {
            (ContinuousObservationSpace::DubinsAircraft, AircraftState::Dubins(dubins_state)) => {
                // Convert DubinsAircraftState to simplified observation vector
                let mut obs = HashMap::new();

                // Get heading from attitude quaternion (yaw angle)
                let euler = dubins_state.spatial.attitude.euler_angles();
                let heading = euler.2; // yaw angle

                // Altitude is negative of the z-component in NED frame
                let altitude = -dubins_state.spatial.position.z;

                // Airspeed from velocity magnitude
                let airspeed = dubins_state.spatial.velocity.magnitude();

                let x = dubins_state.spatial.position.x;
                let y = dubins_state.spatial.position.y;

                obs.insert("x".to_string(), x);
                obs.insert("y".to_string(), y);
                obs.insert("heading".to_string(), heading);
                obs.insert("altitude".to_string(), altitude);
                obs.insert("airspeed".to_string(), airspeed);

                info!("Raw Observation: {:?}", obs);

                obs
            }
            (ContinuousObservationSpace::FullAircraft, AircraftState::Full(full_state)) => {
                // Convert FullAircraftState to simplified observation vector for RL
                let mut obs = HashMap::new();

                // Attitude (roll, pitch, yaw)
                let euler = full_state.spatial.attitude.euler_angles();
                obs.insert("roll".to_string(), euler.0);
                obs.insert("pitch".to_string(), euler.1);
                obs.insert("yaw".to_string(), euler.2);

                // Angular rates (p, q, r)
                obs.insert("p".to_string(), full_state.spatial.angular_velocity.x);
                obs.insert("q".to_string(), full_state.spatial.angular_velocity.y);
                obs.insert("r".to_string(), full_state.spatial.angular_velocity.z);

                // Key flight parameters
                obs.insert("TAS".to_string(), full_state.air_data.true_airspeed);
                obs.insert("alpha".to_string(), full_state.air_data.alpha);
                obs.insert("beta".to_string(), full_state.air_data.beta);
                obs
            }
            _ => HashMap::new(), // Return empty vector for mismatched types
        }
    }
}
