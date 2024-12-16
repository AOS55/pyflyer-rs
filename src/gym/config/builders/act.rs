use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::gym::config::errors::ConfigError;
use crate::gym::ActionSpace;

#[derive(Default)]
pub struct ActionSpaceBuilder {
    act_space: Option<ActionSpace>,
}

impl ActionSpaceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn act_space(mut self, act_type: ActionSpace) -> Self {
        self.act_space = Some(act_type);
        self
    }

    pub fn from_pydict(dict: &Bound<'_, PyDict>) -> PyResult<Self> {
        let mut builder = Self::new();

        if let Some(act_type) = dict.get_item("type")? {
            match act_type.extract()? {
                "ContinuousDubinsAct" => {
                    builder = builder.act_space(ActionSpace::new_dubins());
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
