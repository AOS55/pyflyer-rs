use pyo3::prelude::*;
use flyer::World;
use crate::PyAircraft;
use aerso::types::*;
use std::path::PathBuf;
use std::collections::HashMap;

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
        let ac = &aircraft.aircraft;
        self.world.add_aircraft(ac.clone());
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

    fn step(&mut self, dt: f64) {
        self.world.vehicles[0].step(dt);
        for vid in 0..self.world.vehicles.len() {
            self.world.vehicles[vid].step(dt);
        }
    }

    fn act(&mut self, controls: Vec<HashMap<String, f64>>) {
        for vid in 0..self.world.vehicles.len() {
            self.world.vehicles[vid].act(controls[vid].clone());
        }
    }
    
    fn render(&mut self) -> Vec<u8> {

        let pixmap = self.world.render();
        pixmap.data().to_vec()
    }

    #[getter]
    fn get_vehicles(&mut self) -> PyResult<Vec<PyAircraft>> {

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
    fn get_assets_dir(&self) -> PyResult<PathBuf> {
        Ok(PathBuf::from(&self.world.assets_dir))
    }

    #[getter]
    fn get_camera_pos(&self) -> PyResult<Vec<f64>> {
        let camera_pos = vec![self.world.camera.x, self.world.camera.y, self.world.camera.z];
        Ok(camera_pos)
    }

    #[setter]
    fn set_camera_pos(&mut self, pos: Vec<f64>) {
        self.world.camera.move_camera(pos);
    }
    
}

impl Drop for PyWorld {

    fn drop(&mut self) {
    }

}
