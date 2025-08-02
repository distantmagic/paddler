use std::sync::RwLock;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Notify;

use super::StateDatabase;
use crate::agent_desired_state::AgentDesiredState;

pub struct Memory {
    agent_desired_state: RwLock<AgentDesiredState>,
    update_notifier: Arc<Notify>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            agent_desired_state: RwLock::new(AgentDesiredState::default()),
            update_notifier: Arc::new(Notify::new()),
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

        self.update_notifier.notify_waiters();

        Ok(())
    }
}
