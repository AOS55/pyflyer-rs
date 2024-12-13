use pyo3::prelude::*;

mod gym;
mod utils;

#[pymodule]
fn pyflyer(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<gym::FlyerEnv>()?;
    Ok(())
}
