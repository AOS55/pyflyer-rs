use crate::gym::config::ConfigError;
use numpy::PyArray1;
use pyo3::prelude::*;

#[derive(Debug, Clone)]
pub struct Space {
    pub shape: Vec<usize>,
    pub low: Vec<f64>,
    pub high: Vec<f64>,
}

impl Space {
    pub fn new(shape: Vec<usize>, low: Vec<f64>, high: Vec<f64>) -> Result<Self, ConfigError> {
        if low.len() != shape[0] || high.len() != shape[0] {
            return Err(ConfigError::ValidationError(
                "Space bounds must match shape".into(),
            ));
        }
        Ok(Self { shape, low, high })
    }

    pub fn to_py(&self, py: Python) -> PyResult<(PyObject, PyObject)> {
        let low = PyArray1::from_vec(py, self.low.clone());
        let high = PyArray1::from_vec(py, self.high.clone());
        Ok((low.into_py(py), high.into_py(py))) // update to PyO3 0.23
    }
}
