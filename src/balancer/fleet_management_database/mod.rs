mod file;
mod memory;

use anyhow::Result;
use async_trait::async_trait;

pub use self::file::File;
pub use self::memory::Memory;
use crate::supervisor::llamacpp_desired_state::LlamaCppDesiredState;

#[async_trait]
pub trait FleetManagementDatabase: Send + Sync {
    async fn read_desired_state(&self) -> Result<Option<LlamaCppDesiredState>>;

    async fn store_desired_state(&self, state: &LlamaCppDesiredState) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::NamedTempFile;

    use super::*;

    async fn subtest_store_desired_state<TDatabase: FleetManagementDatabase>(
        db: &TDatabase,
    ) -> Result<()> {
        let desired_state = LlamaCppDesiredState {
            model_path: "test_model_path".to_string(),
        };

        db.store_desired_state(&desired_state).await?;

        let read_state = db.read_desired_state().await?;

        assert_eq!(read_state, Some(desired_state));

        Ok(())
    }

    #[tokio::test]
    async fn test_file_database() -> Result<()> {
        let tempfile = NamedTempFile::new()?;
        let db = File::new(tempfile.path().to_path_buf());

        subtest_store_desired_state(&db).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_database() -> Result<()> {
        let db = Memory::new();

        subtest_store_desired_state(&db).await?;

        Ok(())
    }
}
