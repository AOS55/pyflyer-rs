use pyo3::prelude::*;

use flyer::Aircraft;
use flyer::Trim;

use aerso::types::*;
use std::collections::HashMap;
use std::f64::consts::PI;
use argmin::core::{State, Executor};
use argmin::core::observers::{ObserverMode, SlogLogger};
use argmin::solver::particleswarm::ParticleSwarm;
use nalgebra::dvector;

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
            ("roll".to_string(), &euler.0), ("pitch".to_string(), &euler.1), ("yaw".to_string(), &euler.2),
            ('p'.to_string(), &values[10]), ('q'.to_string(), &values[11]), ('r'.to_string(), &values[12])
        ];
    
        let dict: HashMap<String, f64> = iter
            .into_iter()
            .map(|(k, v)| (k, *v))
            .collect();
        return Ok(dict)
    }

    #[getter]
    fn get_crashed(&self) -> PyResult<bool> { 

        let quaternion = self.aircraft.attitude();
        let euler = quaternion.euler_angles();

        let crashed = if self.aircraft.position()[2] > -5.0 {
            if self.aircraft.velocity()[0] > 200.0 {
                true
            } else if self.aircraft.velocity()[2] > 60.0 {
                true
            } else if (-5.0 * PI/180.0) > euler.0 {
                true
            } else if euler.0 > (30.0 * (PI/180.0)) {
                true
            } else {
                false
            }
        } else {
            false
        };

        Ok(crashed)
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

    fn trim(&mut self, alt: f64, airspeed: f64, n_iters: u64) -> PyResult<Vec<f64>> {
        
        let cost = Trim {
            alt,
            airspeed
        };
    
        let solver = ParticleSwarm::new((dvector![-20.0 * (PI/180.0), -4.0 * (PI/180.0), 0.0], dvector![20.0 * (PI/180.0), 4.0 * (PI/180.0), 1.0]), 40);
            
        let res = Executor::new(cost, solver)
            .configure(|state| state.max_iters(n_iters))
            .add_observer(SlogLogger::term(), ObserverMode::Always)
            .run();
            
        let trim_result = match res {
            Ok(trim_result) => trim_result,
            Err(error) => panic!("ArgMin Error: {}", error)
        };
            
        let best = trim_result.state().get_best_param().unwrap();
        let best_pos = vec![best.position[0], best.position[1], best.position[2]];

        Ok(best_pos)

    }

}
