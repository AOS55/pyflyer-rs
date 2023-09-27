use pyo3::prelude::*;


use std::path::Path;

use flyer::Aircraft;
use flyer::World;

use crate::PyAircraft;

use aerso::types::*;

#[pyclass(name="World", unsendable)]
pub struct PyWorld {
    world: World
}

#[pymethods]
impl PyWorld {

    #[new]
    fn new() -> Self {
        
        let w = World::default();

        let local_resources = Path::new("resources");
        w.ctx.fs.mount(local_resources, true);
        println!("{:?}", w.ctx.fs.resources_dir());
        
        Self {world: w} 
    
    }

    fn add_aircraft(
        &mut self,
        aircraft: &PyAircraft
    ) {
        
        let name = &aircraft.aircraft.name;
        let pos = aircraft.aircraft.position();
        let vel = aircraft.aircraft.velocity();
        let att = aircraft.aircraft.attitude();
        let rates = aircraft.aircraft.rates();
        let ac = Aircraft::new(&name, pos, vel, att, rates);
        
        self.world.add_aircraft(ac);
    }

    fn create_map(
        &mut self,
        seed: u64,
        area: Option<Vec<usize>>,
        scaling: Option<f32>,
        water_present: Option<bool>
    ) {
        self.world.create_map(seed, area, scaling, water_present);
    }

    fn render(&mut self) {
        self.world.render();
    }

    fn get_image(&mut self) -> Vec<u8> {
        self.world.get_image()
    }

    #[getter]
    fn get_vehicles(&self) -> PyResult<Vec<PyAircraft>> {
        let vehicles = self.world.vehicles.iter().map(|ac| PyAircraft{aircraft: ac.clone()}).collect();
        Ok(vehicles)
    }
    
}

impl Drop for PyWorld {

    fn drop(&mut self) {
        println!("Calling the destructor!");
        self.world.ctx.quit_requested = true;
    }

}
