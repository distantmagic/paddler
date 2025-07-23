use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::process::Child;

use crate::cleanable::Cleanable;

#[derive(Debug, Default)]
pub struct AgentInstanceCollection {
    pub instances: DashMap<String, Child>,
}

#[async_trait]
impl Cleanable for AgentInstanceCollection {
    async fn cleanup(&mut self) -> Result<()> {
        for mut agent in self.instances.iter_mut() {
            agent.value_mut().kill().await?;
        }

        self.instances.clear();

        Ok(())
    }
}
