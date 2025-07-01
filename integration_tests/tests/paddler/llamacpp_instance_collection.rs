use anyhow::Result;
use anyhow::anyhow;
use async_trait::async_trait;
use dashmap::DashMap;

use crate::cleanable::Cleanable;
use crate::llamacpp_instance::LlamaCppInstance;

#[derive(Debug, Default)]
pub struct LlamaCppInstanceCollection {
    pub instances: DashMap<String, LlamaCppInstance>,
    pub last_llamacpp_port_offset: u16,
}

impl LlamaCppInstanceCollection {
    pub fn llamacpp_port(&self, llamacpp_name: &str) -> Result<u16> {
        if let Some(llama) = self.instances.get(llamacpp_name) {
            Ok(llama.port)
        } else {
            Err(anyhow!("LlamaCpp instance {} not found", llamacpp_name))
        }
    }

    pub fn next_llamacpp_port(&mut self) -> u16 {
        let port = 8000 + self.last_llamacpp_port_offset;

        self.last_llamacpp_port_offset += 1;

        port
    }

    pub async fn kill(&self, llamacpp_name: &str) -> Result<()> {
        if let Some((_, mut llamacpp)) = self.instances.remove(llamacpp_name) {
            Ok(llamacpp.kill().await?)
        } else {
            Err(anyhow!("LlamaCpp instance {} not found", llamacpp_name))
        }
    }
}

#[async_trait]
impl Cleanable for LlamaCppInstanceCollection {
    async fn cleanup(&mut self) -> Result<()> {
        for mut llama in self.instances.iter_mut() {
            llama.cleanup().await?;
        }

        self.instances.clear();
        self.last_llamacpp_port_offset = 0;

        Ok(())
    }
}
