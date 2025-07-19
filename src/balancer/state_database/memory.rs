use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;

use super::StateDatabase;
use crate::agent_desired_state::AgentDesiredState;

pub struct Memory {
    desired_state: RwLock<Option<AgentDesiredState>>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            desired_state: RwLock::new(None),
        }
    }
}

#[async_trait]
impl StateDatabase for Memory {
    async fn read_desired_state(&self) -> Result<Option<AgentDesiredState>> {
        Ok(self.desired_state.read().await.clone())
    }

    async fn store_desired_state(&self, state: &AgentDesiredState) -> Result<()> {
        {
            let mut desired_state = self.desired_state.write().await;

            *desired_state = Some(state.clone());
        }

        Ok(())
    }
}
