use pyo3::exceptions::PyValueError;
use pyo3::PyErr;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    InvalidPhysicsModel(String),
    InvalidAircraftType(String),
    InvalidParameter { name: String, value: String },
    MissingRequired(String),
    ValidationError(String),
    PythonError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidPhysicsModel(msg) => write!(f, "Invalid physics model: {}", msg),
            ConfigError::InvalidAircraftType(msg) => write!(f, "Invalid aircraft type: {}", msg),
            ConfigError::InvalidParameter { name, value } => {
                write!(f, "Invalid parameter '{}' with value '{}'", name, value)
            }
            ConfigError::MissingRequired(name) => write!(f, "Missing required parameter: {}", name),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ConfigError::PythonError(msg) => write!(f, "Python error: {}", msg),
        }
    }
}

impl From<ConfigError> for PyErr {
    fn from(err: ConfigError) -> PyErr {
        PyValueError::new_err(err.to_string())
    }
}

impl From<PyErr> for ConfigError {
    fn from(err: PyErr) -> ConfigError {
        ConfigError::PythonError(err.to_string())
    }
}
