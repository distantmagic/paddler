mod file;
mod memory;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::Notify;

pub use self::file::File;
pub use self::memory::Memory;
use crate::chat_template::ChatTemplate;
use crate::chat_template_head::ChatTemplateHead;
use crate::agent_desired_state::AgentDesiredState;

#[async_trait]
pub trait StateDatabase: Send + Sync {
    fn get_update_notifier(&self) -> Arc<Notify>;

    async fn list_chat_template_heads(&self) -> Result<Vec<ChatTemplateHead>>;

    async fn read_agent_desired_state(&self) -> Result<AgentDesiredState>;

    async fn read_chat_template(&self, id: String) -> Result<Option<ChatTemplate>>;

    async fn store_agent_desired_state(&self, state: &AgentDesiredState) -> Result<()>;

    async fn store_chat_template(&self, chat_template: &ChatTemplate) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::NamedTempFile;
    use chrono::Utc;

    use super::*;
    use crate::agent_desired_model::AgentDesiredModel;
    use crate::inference_parameters::InferenceParameters;

    async fn subtest_store_chat_template<TDatabase: StateDatabase>(db: &TDatabase) -> Result<()> {
        let chat_template = ChatTemplate {
            content: "test_content".to_string(),
            id: "test_id".to_string(),
            name: "test_name".to_string(),
            updated_at: Utc::now(),
        };

        db.store_chat_template(&chat_template).await?;

        let read_template = db.read_chat_template("test_id".to_string()).await?.expect("Chat template should be found");

        assert_eq!(read_template.content, chat_template.content);

        let template_heads = db.list_chat_template_heads().await?;

        assert_eq!(template_heads.len(), 1);

        assert_eq!(template_heads[0].id, chat_template.id);
        assert_eq!(template_heads[0].name, chat_template.name);

        Ok(())
    }

    async fn subtest_store_desired_state<TDatabase: StateDatabase>(db: &TDatabase) -> Result<()> {
        let desired_state = AgentDesiredState {
            inference_parameters: InferenceParameters::default(),
            model: AgentDesiredModel::Local("test_model_path".to_string()),
            override_chat_template: None,
        };

        db.store_agent_desired_state(&desired_state).await?;

        let read_state = db.read_agent_desired_state().await?;

        assert_eq!(read_state.model, desired_state.model);

        Ok(())
    }

    #[tokio::test]
    async fn test_file_database() -> Result<()> {
        let tempfile = NamedTempFile::new()?;
        let db = File::new(tempfile.path().to_path_buf());

        subtest_store_desired_state(&db).await?;
        subtest_store_chat_template(&db).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_database() -> Result<()> {
        let db = Memory::new();

        subtest_store_desired_state(&db).await?;
        subtest_store_chat_template(&db).await?;

        Ok(())
    }
}
