use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;

use super::FleetManagementDatabase;
use crate::llamacpp::llamacpp_state::LlamaCppState;

pub struct Memory {
    desired_state: RwLock<Option<LlamaCppState>>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            desired_state: RwLock::new(None),
        }
    }
}

#[async_trait]
impl FleetManagementDatabase for Memory {
    async fn read_desired_state(&self) -> Result<Option<LlamaCppState>> {
        let desired_state = { self.desired_state.read().await.clone() };

        Ok(desired_state)
    }

    async fn store_desired_state(&self, state: &LlamaCppState) -> Result<()> {
        {
            let mut desired_state = self.desired_state.write().await;

            *desired_state = Some(state.clone());
        }

        Ok(())
    }
}
