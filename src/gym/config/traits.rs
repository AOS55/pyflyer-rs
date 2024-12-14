use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Trait for types that can be constructed from Python dictionaries
pub trait FromPyDict: Sized {
    fn from_pydict(dict: &PyDict) -> PyResult<Self>;
}
