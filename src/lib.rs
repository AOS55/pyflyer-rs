use pyo3::prelude::*;

mod gym;
pub mod utils;

#[pymodule]
fn pyflyer(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<gym::FlyerEnv>()?;
    Ok(())
}
