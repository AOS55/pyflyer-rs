use pyo3::prelude::*;

mod aircraft;
use aircraft::PyAircraft;

mod world;
use world::PyWorld;

#[pymodule]
#[pyo3(name = "pyflyer")]
fn my_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyAircraft>()?;
    m.add_class::<PyWorld>()?;
    Ok(())
}
