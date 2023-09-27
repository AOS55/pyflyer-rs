use pyo3::prelude::*;

use flyer::Aircraft;
use aerso::types::*;

use std::collections::HashMap;

#[pyclass(name="Aircraft", unsendable)]
pub struct PyAircraft {
    pub aircraft: Aircraft
}

#[pymethods]
impl PyAircraft {

    #[getter]
    fn get_position(&self) -> PyResult<[f64;3]> {
        Ok(self.aircraft.position().into())
    }

    #[getter]
    fn get_velocity(&self) -> PyResult<[f64;3]> {
        Ok(self.aircraft.velocity().into())
    }

    #[getter]
    fn get_attitude(&self) -> PyResult<[f64;4]> {
        let q = self.aircraft.attitude();
        Ok([q.i, q.j, q.k, q.w])
    }

    #[getter]
    fn get_rates(&self) -> PyResult<[f64;3]> {
        Ok(self.aircraft.rates().into())
    }

    #[getter]
    fn get_names(&self) -> PyResult<String> {
        Ok(self.aircraft.name.clone())
    }

    #[getter]
    fn get_states(&self) -> PyResult<[f64;13]> {
        Ok(self.aircraft.statevector().into())
    }

    #[getter]
    fn get_dict(&self) -> PyResult<HashMap<String, f64>> {
        let values = self.aircraft.statevector();
        let quaternion = self.aircraft.attitude();
        let euler = quaternion.euler_angles();
        let iter = vec![
            ('x'.to_string(), &values[0]), ('y'.to_string(), &values[1]), ('z'.to_string(), &values[2]),
            ('u'.to_string(), &values[3]), ('v'.to_string(), &values[4]), ('w'.to_string(), &values[5]),
            ("pitch".to_string(), &euler.0), ("roll".to_string(), &euler.1), ("yaw".to_string(), &euler.2),
            ('p'.to_string(), &values[9]), ('q'.to_string(), &values[10]), ('r'.to_string(), &values[11])
        ];
    
        let dict: HashMap<String, f64> = iter
            .into_iter()
            .map(|(k, v)| (k, *v))
            .collect();
        return Ok(dict)
    }

    #[new]
    fn new(
        aircraft_name: Option<&str>,
        initial_position: Option<Vec<f64>>,
        initial_velocity: Option<Vec<f64>>,
        initial_attitude: Option<Vec<f64>>,
        initial_rates: Option<Vec<f64>>
    ) -> Self {
        
        let aircraft_name = if let Some(name) = aircraft_name {
            name
        } else{
            "TO"
        };

        let initial_position = if let Some(pos) = initial_position {
            Vector3::new(pos[0], pos[1], pos[2])
        } else{
            Vector3::zeros()
        };

        let initial_velocity = if let Some(vel) = initial_velocity {
            Vector3::new(vel[0], vel[1], vel[2])
        } else{
            Vector3::zeros()
        };

        let initial_attitude = if let Some(att) = initial_attitude {
            UnitQuaternion::from_euler_angles(att[0], att[1], att[2])
        } else{
            UnitQuaternion::identity()
        };

        let initial_rates = if let Some(rate) = initial_rates {
            Vector3::new(rate[0], rate[1], rate[2])
        } else{
            Vector3::zeros()
        };

        Self {
            aircraft: Aircraft::new(
                aircraft_name,
                initial_position,
                initial_velocity,
                initial_attitude,
                initial_rates
            )
        }
    }

    fn reset(
        &mut self,
        pos: Vec<f64>,
        heading: f64,
        airspeed: f64,
        aircraft_name: Option<&str>,
    ) {

        let aircraft_name = if let Some(name) = aircraft_name {
            name
        } else{
            "TO"
        };

        let pos = Vector3::new(pos[0], pos[1], pos[2]);
        let velocity = Vector3::new(airspeed, 0.0, 0.0);
        let attitude = UnitQuaternion::from_euler_angles(0.0, 0.0, heading);
        let rates = Vector3::zeros();

        self.aircraft = Aircraft::new(
            aircraft_name,
            pos,
            velocity,
            attitude,
            rates
        );
    }

    fn step(&mut self, dt: f64, input: Vec<f64>) {
        self.aircraft.aff_body.step(dt, &input);
    }
}
