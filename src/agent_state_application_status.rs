use anyhow::anyhow;
use anyhow::Result;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AgentStateApplicationStatus {
    Applied,
    AttemptedAndNotAppliable,
    AttemptedAndRetrying,
    Fresh,
    Stuck,
}

impl AgentStateApplicationStatus {
    pub fn from_code(code: i32) -> Result<Self> {
        match code {
            0 => Ok(AgentStateApplicationStatus::Applied),
            1 => Ok(AgentStateApplicationStatus::AttemptedAndNotAppliable),
            2 => Ok(AgentStateApplicationStatus::AttemptedAndRetrying),
            3 => Ok(AgentStateApplicationStatus::Fresh),
            4 => Ok(AgentStateApplicationStatus::Stuck),
            _ => Err(anyhow!("Invalid AgentStateApplicationStatus code: {}", code)),
        }
    }

    pub fn should_try_to_apply(&self) -> bool {
        match self {
            AgentStateApplicationStatus::Applied => false,
            AgentStateApplicationStatus::AttemptedAndNotAppliable => false,
            AgentStateApplicationStatus::AttemptedAndRetrying => true,
            AgentStateApplicationStatus::Fresh => true,
            AgentStateApplicationStatus::Stuck => true,
        }
    }

    pub fn to_code(&self) -> i32 {
        match self {
            AgentStateApplicationStatus::Applied => 0,
            AgentStateApplicationStatus::AttemptedAndNotAppliable => 1,
            AgentStateApplicationStatus::AttemptedAndRetrying => 2,
            AgentStateApplicationStatus::Fresh => 3,
            AgentStateApplicationStatus::Stuck => 4,
        }
    }
}
