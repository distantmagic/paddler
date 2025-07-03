use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use dashmap::DashMap;

use crate::cleanable::Cleanable;
use crate::supervisor_instance::SupervisorInstance;

#[derive(Debug, Default)]
pub struct SupervisorInstanceCollection {
    pub instances: DashMap<String, SupervisorInstance>,
}

impl SupervisorInstanceCollection {
    pub fn llamacpp_port(&self, supervisor_name: &str) -> Result<u16> {
        if let Some(supervisor) = self.instances.get(supervisor_name) {
            Ok(supervisor.llamacpp_listen_port)
        } else {
            Err(anyhow!("Supervisor instance {supervisor_name} not found"))
        }
    }
}

#[async_trait]
impl Cleanable for SupervisorInstanceCollection {
    async fn cleanup(&mut self) -> Result<()> {
        for mut agent in self.instances.iter_mut() {
            agent.value_mut().child.kill().await?;
        }

        self.instances.clear();

        Ok(())
    }
}
