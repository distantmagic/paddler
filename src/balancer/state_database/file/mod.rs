use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use log::warn;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

use super::StateDatabase;
use crate::agent_desired_state::AgentDesiredState;

pub struct File {
    path: PathBuf,
    write_lock: RwLock<()>,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        File {
            path,
            write_lock: RwLock::new(()),
        }
    }

    async fn read_desired_state_from_file(&self) -> Result<Option<AgentDesiredState>> {
        match fs::read_to_string(&self.path).await {
            Ok(content) => {
                if content.is_empty() {
                    return Ok(None);
                }

                let state: AgentDesiredState = serde_json::from_str(&content)
                    .context(format!("Unable to parse database file contents: '{}'. Either that is not a valid database file, or this version of Paddler is incompatible with it.", self.path.display()))?;

                Ok(Some(state))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                warn!(
                    "State database file not found; trying to store the default state: '{}'",
                    self.path.display()
                );

                self.store_desired_state(&AgentDesiredState::default())
                    .await
                    .context("Failed to store default state after file not found")?;

                Ok(None)
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl StateDatabase for File {
    async fn read_desired_state(&self) -> Result<Option<AgentDesiredState>> {
        self.read_desired_state_from_file()
            .await
            .context("Unable to read state from file")
    }

    async fn store_desired_state(&self, state: &AgentDesiredState) -> Result<()> {
        let _lock = self.write_lock.write().await;

        let serialized_state = serde_json::to_string_pretty(state)?;
        let mut file = fs::File::create(&self.path).await?;

        file.write_all(serialized_state.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }
}
