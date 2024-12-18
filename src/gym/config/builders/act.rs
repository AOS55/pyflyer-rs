use crate::gym::config::ConfigError;
use crate::gym::ActionSpace;

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
        Self {
            act_space: Some(ActionSpace::new_continuous_dubins()),
        }
    }

    pub fn act_space(mut self, act_space: ActionSpace) -> Self {
        self.act_space = Some(act_space);
        self
    }

    pub fn build(self) -> Result<ActionSpace, ConfigError> {
        self.act_space.ok_or(ConfigError::MissingActionSpace)
    }
}
