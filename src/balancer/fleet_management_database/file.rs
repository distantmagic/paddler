use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

use super::FleetManagementDatabase;
use super::Memory;
use crate::llamacpp::llamacpp_state::LlamaCppState;

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

    async fn read_desired_state_from_file(&self) -> Result<Option<LlamaCppState>> {
        match fs::read_to_string(&self.path).await {
            Ok(content) => {
                let state: LlamaCppState = serde_json::from_str(&content)?;

                Ok(Some(state))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl FleetManagementDatabase for File {
    async fn read_desired_state(&self) -> Result<Option<LlamaCppState>> {
        match self.cached_state.read_desired_state().await? {
            Some(state) => Ok(Some(state)),
            None => {
                let state = self.read_desired_state_from_file().await?;

                if let Some(ref state) = state {
                    self.cached_state.store_desired_state(state).await?;
                }

                Ok(state)
            }
        }
    }

    async fn store_desired_state(&self, state: &LlamaCppState) -> Result<()> {
        let _lock = self.write_lock.write().await;

        let serialized_state = serde_json::to_string_pretty(state)?;
        let mut file = fs::File::create(&self.path).await?;

        file.write_all(serialized_state.as_bytes()).await?;
        file.sync_all().await?;

        self.cached_state.store_desired_state(state).await?;

        Ok(())
    }
}
