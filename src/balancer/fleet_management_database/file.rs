use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use super::FleetManagementDatabase;
use crate::llamacpp::llamacpp_state::LlamaCppState;

pub struct File {
    path: PathBuf,
    write_lock: Mutex<()>,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        File {
            path,
            write_lock: Mutex::new(()),
        }
    }
}

#[async_trait]
impl FleetManagementDatabase for File {
    async fn store_desired_state(&self, state: &LlamaCppState) -> Result<()> {
        let serialized_state = serde_json::to_string_pretty(state)?;
        let mut file = fs::File::create(&self.path).await?;

        file.write_all(serialized_state.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }
}
