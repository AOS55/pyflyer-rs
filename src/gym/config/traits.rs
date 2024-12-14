use flyer::components::AircraftConfig;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand_chacha::ChaCha8Rng;

use crate::gym::config::ConfigError;

/// Trait for types that can be constructed from Python dictionaries
pub trait FromPyDict: Sized {
    fn from_pydict(dict: &PyDict) -> PyResult<Self>;
}
