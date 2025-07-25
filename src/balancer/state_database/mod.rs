mod file;
mod memory;

use anyhow::Result;
use async_trait::async_trait;

pub use self::file::File;
pub use self::memory::Memory;
use crate::agent_desired_state::AgentDesiredState;

#[async_trait]
pub trait StateDatabase: Send + Sync {
    async fn read_desired_state(&self) -> Result<Option<AgentDesiredState>>;

    async fn store_desired_state(&self, state: &AgentDesiredState) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::agent_desired_model::AgentDesiredModel;
    use crate::model_parameters::ModelParameters;

    async fn subtest_store_desired_state<TDatabase: StateDatabase>(db: &TDatabase) -> Result<()> {
        let desired_state = AgentDesiredState {
            model: AgentDesiredModel::Local("test_model_path".to_string()),
            model_parameters: ModelParameters::default(),
        };

        db.store_desired_state(&desired_state).await?;

        let read_state = db.read_desired_state().await?;

        assert_eq!(read_state.unwrap().model, desired_state.model);

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
