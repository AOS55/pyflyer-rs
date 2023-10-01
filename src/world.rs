use pyo3::prelude::*;
use flyer::Aircraft;
use flyer::World;
use crate::PyAircraft;
use aerso::types::*;
use std::path::PathBuf;

#[pyclass(name="World", unsendable)]
pub struct PyWorld {
    world: World
}

#[pymethods]
impl PyWorld {

    #[new]
    fn new() -> Self {
        
        let w = World::default();
        
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

    fn render(&mut self) -> Vec<u8> {
       let pixmap = self.world.render();
    //    pixmap.encode_png().unwrap()
        pixmap.data().to_vec()
    }

    // fn get_image(&mut self) -> Vec<u8> {
    //     self.world.get_image()
    // }

    #[getter]
    fn get_vehicles(&self) -> PyResult<Vec<PyAircraft>> {
        let vehicles = self.world.vehicles.iter().map(|ac| PyAircraft{aircraft: ac.clone()}).collect();
        Ok(vehicles)
    }

    #[setter]
    fn set_screen_dim(&mut self, dim: Vec<f32>) {
        self.world.set_screen_dims(dim[0], dim[1])
    }

    #[getter]
    fn get_screen_dim(&mut self) -> PyResult<Vec<f32>> {
        let dim = self.world.screen_dims;
        Ok(vec![dim[0], dim[1]])
    }

    #[getter]
    fn get_screen_width(&mut self) -> PyResult<f32> {
        Ok(self.world.screen_dims[0])
    }

    #[getter]
    fn get_screen_height(&mut self) -> PyResult<f32> {
        Ok(self.world.screen_dims[0])
    }

    #[setter]
    fn set_assets_dir(&mut self, asset_dir: PathBuf) {
        self.world.set_assets_dir(asset_dir)
    }

    #[getter]
    fn get_assets_dir(&mut self) -> PyResult<PathBuf> {
        Ok(PathBuf::from(&self.world.assets_dir))
    }
    
}

impl Drop for PyWorld {

    fn drop(&mut self) {
    }

}
