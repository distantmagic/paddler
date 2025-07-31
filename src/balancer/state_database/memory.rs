use std::sync::RwLock;

use anyhow::Result;
use async_trait::async_trait;

use super::StateDatabase;
use crate::agent_desired_state::AgentDesiredState;

pub struct Memory {
    agent_desired_state: RwLock<AgentDesiredState>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            agent_desired_state: RwLock::new(AgentDesiredState::default()),
        }
    }
}

#[async_trait]
impl StateDatabase for Memory {
    async fn read_agent_desired_state(&self) -> Result<AgentDesiredState> {
        Ok(self
            .agent_desired_state
            .read()
            .expect("Failed to acquire read lock")
            .clone())
    }

    async fn store_agent_desired_state(&self, state: &AgentDesiredState) -> Result<()> {
        {
            let mut agent_desired_state = self
                .agent_desired_state
                .write()
                .expect("Failed to acquire write lock");

            *agent_desired_state = state.clone();
        }

        Ok(())
    }
}
