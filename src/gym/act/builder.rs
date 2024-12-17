use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::config::ConfigError;
use crate::gym::ActionSpace;

pub struct ActionSpaceBuilder {
    act_space: Option<ActionSpace>,
}

impl Default for ActionSpaceBuilder {
    fn default() -> Self {
        Self {
            act_space: Some(ActionSpace::new_dubins()),
        }
    }
}

impl ActionSpaceBuilder {
    pub fn new() -> Self {
        Self {
            act_space: Some(ActionSpace::new_dubins()),
        }
    }

    pub fn act_space(mut self, act_space: ActionSpace) -> Self {
        self.act_space = Some(act_space);
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        // ToDo implement method to take aircraft and create obs based on this action (for full aircraft)
        if let Some(act_type) = dict.get_item("type")? {
            match act_type.extract()? {
                "Continuous" => {
                    builder = builder.act_space(ActionSpace::new_dubins());
                }
                "Discrete" => {
                    builder = builder.act_space(ActionSpace::new_discrete_dubins());
                }
                _ => return Err(ConfigError::InvalidActionType(act_type.to_string()).into()),
            }
        }

        Ok(builder)
    }

    pub fn build(self) -> Result<ActionSpace, ConfigError> {
        self.act_space.ok_or(ConfigError::MissingActionSpace)
    }
}
