use flyer::components::AircraftState;
use numpy::PyArray1;
use pyo3::prelude::*;

pub trait FromAircraft {
    fn from_aircraft<'py>(
        &self,
        py: Python<'py>,
        state: &AircraftState,
    ) -> PyResult<Bound<'py, PyAny>>;
}

#[derive(Debug, Clone, Copy)]
pub enum ContinuousObservationSpace {
    DubinsAircraft,
    FullAircraft,
}

#[derive(Copy, Debug, Clone)]
pub enum ObservationSpace {
    Continuous(ContinuousObservationSpace),
}

impl Default for ObservationSpace {
    fn default() -> Self {
        ObservationSpace::Continuous(ContinuousObservationSpace::DubinsAircraft)
    }
}

impl FromAircraft for ObservationSpace {
    fn from_aircraft<'py>(
        &self,
        py: Python<'py>,
        state: &AircraftState,
    ) -> PyResult<Bound<'py, PyAny>> {
        match self {
            ObservationSpace::Continuous(continuous) => continuous.from_aircraft(py, state),
        }
    }
}

impl FromAircraft for ContinuousObservationSpace {
    fn from_aircraft<'py>(
        &self,
        py: Python<'py>,
        state: &AircraftState,
    ) -> PyResult<Bound<'py, PyAny>> {
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
                let array = PyArray1::from_vec(py, obs);
                Ok(array.into_any())
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
                let array = PyArray1::from_vec(py, obs);
                Ok(array.into_any())
            }
            _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Observation space type does not match aircraft state type",
            )),
        }
    }
}
