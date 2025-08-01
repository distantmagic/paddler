mod schema;

use std::sync::Arc;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use log::warn;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::sync::Notify;

use crate::chat_template_head::ChatTemplateHead;
use self::schema::Schema;
use super::StateDatabase;
use crate::agent_desired_state::AgentDesiredState;
use crate::chat_template::ChatTemplate;

pub struct File {
    path: PathBuf,
    update_notifier: Arc<Notify>,
    write_lock: RwLock<()>,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        File {
            path,
            update_notifier: Arc::new(Notify::new()),
            write_lock: RwLock::new(()),
        }
    }

    async fn read_schema_from_file(&self) -> Result<Schema> {
        match fs::read_to_string(&self.path).await {
            Ok(content) => {
                if content.is_empty() {
                    return self.store_default_schema().await;
                }

                let schema: Schema = serde_json::from_str(&content).context(format!("Unable to parse database file contents: '{}'. Either that is not a valid database file, or this version of Paddler is incompatible with it.", self.path.display()))?;

                Ok(schema)
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                warn!(
                    "State database file not found; trying to store the default state: '{}'",
                    self.path.display()
                );

                self.store_default_schema().await
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn store_default_schema(&self) -> Result<Schema> {
        let schema = Schema::default();

        self.store_schema(&schema)
            .await
            .context("Failed to store default state")?;

        Ok(schema)
    }

    async fn store_schema(&self, schema: &Schema) -> Result<()> {
        let _lock = self.write_lock.write().await;

        let serialized_schema = serde_json::to_string_pretty(schema)?;
        let mut file = fs::File::create(&self.path).await?;

        file.write_all(serialized_schema.as_bytes()).await?;
        file.sync_all().await?;

        self.update_notifier.notify_waiters();

        Ok(())
    }

    async fn update_schema<TFunction>(&self, modifier: TFunction) -> Result<()>
    where
        TFunction: FnOnce(&mut Schema),
    {
        let mut schema = self
            .read_schema_from_file()
            .await
            .context("Unable to read current state from file")?;

        modifier(&mut schema);

        self.store_schema(&schema).await
    }
}

#[async_trait]
impl StateDatabase for File {
    async fn delete_chat_template(&self, id: String) -> Result<()> {
        self.update_schema(|schema| {
            schema.chat_templates.remove(&id);
        }).await
    }

    fn get_update_notifier(&self) -> Arc<Notify> {
        self.update_notifier.clone()
    }

    async fn list_chat_template_heads(&self) -> Result<Vec<ChatTemplateHead>> {
        Ok(self
            .read_schema_from_file()
            .await
            .context("Unable to read chat templates from file")?
            .chat_templates
            .values()
            .map(|template| template.to_head())
            .collect())
    }

    async fn read_chat_template(&self, id: String) -> Result<Option<ChatTemplate>> {
        Ok(self
            .read_schema_from_file()
            .await
            .context("Unable to read chat template from file")?
            .chat_templates
            .get(&id)
            .cloned())
    }

    async fn read_agent_desired_state(&self) -> Result<AgentDesiredState> {
        Ok(self
            .read_schema_from_file()
            .await
            .context("Unable to read state from file")?
            .agent_desired_state
            .clone())
    }

    async fn store_agent_desired_state(&self, agent_desired_state: &AgentDesiredState) -> Result<()> {
        self.update_schema(|schema| {
            schema.agent_desired_state = agent_desired_state.clone();
        }).await
    }

    async fn store_chat_template(&self, chat_template: &ChatTemplate) -> Result<()> {
        self.update_schema(|schema| {
            schema.chat_templates.insert(chat_template.id.clone(), chat_template.clone());
        }).await
    }
}
