use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

use super::FleetManagementDatabase;
use super::Memory;
use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;

pub struct File {
    cached_state: Memory,
    path: PathBuf,
    write_lock: RwLock<()>,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        File {
            cached_state: Memory::new(),
            path,
            write_lock: RwLock::new(()),
        }
    }

    async fn read_desired_state_from_file(&self) -> Result<Option<LlamaCppDesiredState>> {
        match fs::read_to_string(&self.path).await {
            Ok(content) => {
                if content.is_empty() {
                    return Ok(None);
                }

                let state: LlamaCppDesiredState = serde_json::from_str(&content)
                    .context(format!("Unable to parse file contents: '{content}'"))?;

                Ok(Some(state))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl FleetManagementDatabase for File {
    async fn read_desired_state(&self) -> Result<Option<LlamaCppDesiredState>> {
        match self.cached_state.read_desired_state().await? {
            Some(state) => Ok(Some(state)),
            None => {
                let state = self
                    .read_desired_state_from_file()
                    .await
                    .context("Unable to read state from file")?;

                if let Some(ref state) = state {
                    self.cached_state
                        .store_desired_state(state)
                        .await
                        .context("Unable to store state to file")?;
                }

                Ok(state)
            }
        }
    }

    async fn store_desired_state(&self, state: &LlamaCppDesiredState) -> Result<()> {
        let _lock = self.write_lock.write().await;

        let serialized_state = serde_json::to_string_pretty(state)?;
        let mut file = fs::File::create(&self.path).await?;

        file.write_all(serialized_state.as_bytes()).await?;
        file.sync_all().await?;

        self.cached_state.store_desired_state(state).await?;

        Ok(())
    }
}
