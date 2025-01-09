use crate::gym::config::ConfigError;
use crate::gym::ActionSpace;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSpaceBuilder {
    act_space: Option<ActionSpace>,
}

impl Default for ActionSpaceBuilder {
    fn default() -> Self {
        Self {
            act_space: Some(ActionSpace::new_continuous_dubins()),
        }
    }
}

impl ActionSpaceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom action space
    pub fn act_space(mut self, act_space: ActionSpace) -> Self {
        self.act_space = Some(act_space);
        self
    }

    /// Builds the action space, returning an error if no action space was set
    pub fn build(self) -> Result<ActionSpace, ConfigError> {
        self.act_space.ok_or(ConfigError::MissingActionSpace)
    }
}
