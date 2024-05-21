use pyo3::prelude::*;
use pyo3::exceptions::PyAttributeError;
use flyer::World;
use crate::PyAircraft;
use std::path::PathBuf;
use std::collections::HashMap;
use glam::Vec2;

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

    fn update_aircraft(&mut self, aircraft: &PyAircraft, id: usize) {
        let ac = &aircraft.aircraft;
        self.world.update_aircraft(ac.clone(), id);
    }

    fn add_runway(
        &mut self,
        runway_position: Option<Vec<f32>>,
        runway_width: Option<f32>,
        runway_length: Option<f32>,
        runway_heading: Option<f32>,
    ) {
        self.world.create_runway();

        match runway_position {
            Some(runway_position) => {
                self.world.runway.as_mut().unwrap().pos = Vec2::new(runway_position[0], runway_position[1]);
            },
            None => ()
        }

        match runway_width {
            Some(runway_width) => {
                self.world.runway.as_mut().unwrap().dims[0] = runway_width;
            },
            None => ()
        }

        match runway_length {
            Some(runway_length) => {
                self.world.runway.as_mut().unwrap().dims[1] = runway_length;
            },
            None => ()
        }

        match runway_heading {
            Some(runway_heading) => {
                self.world.runway.as_mut().unwrap().heading = runway_heading;
            },
            None => ()
        }

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

    fn point_on_runway(&self, point: Vec<f32>) -> PyResult<bool> {
    
        match &self.world.runway {
            Some(runway) => {
                Ok(runway.on_runway(Vec2::new(point[0], point[1])))
            },
            None => {
                Err(PyAttributeError::new_err("No runway in world, call `add_runway` first"))
            }
        }
    }

    fn touchdown_points(&self) -> PyResult<HashMap<String, Vec<f32>>> {

        match &self.world.runway{
            Some(runway) => {
                Ok(runway.approach_points())
            },
            None => {
                Err(PyAttributeError::new_err("No runway in world, call `add_runway` first"))
            }
        }
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

    #[setter]
    fn set_terrain_data_dir(&mut self, terrain_data_dir: PathBuf) {
        self.world.set_terrain_data_dir(terrain_data_dir)
    }

    #[getter]
    fn get_terrain_data_dir(&self) -> PyResult<PathBuf> {
        Ok(PathBuf::from(&self.world.terrain_data_dir))
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

    #[setter]
    fn set_render_type(&mut self, render_type: String){
        self.world.render_type = render_type;
    }

    #[getter]
    fn get_render_type(&self) -> PyResult<String> {
        Ok(self.world.render_type.to_string())
    }
    
}

impl Drop for PyWorld {

    fn drop(&mut self) {
    }

}
