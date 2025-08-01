use std::sync::RwLock;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::Notify;

use crate::chat_template::ChatTemplate;
use crate::chat_template_head::ChatTemplateHead;
use super::StateDatabase;
use crate::agent_desired_state::AgentDesiredState;

pub struct Memory {
    agent_desired_state: RwLock<AgentDesiredState>,
    chat_templates: DashMap<String, ChatTemplate>,
    update_notifier: Arc<Notify>,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            agent_desired_state: RwLock::new(AgentDesiredState::default()),
            chat_templates: DashMap::new(),
            update_notifier: Arc::new(Notify::new()),
        }
    }
}

#[async_trait]
impl StateDatabase for Memory {
    async fn delete_chat_template(&self, id: String) -> Result<()> {
        if self.chat_templates.remove(&id).is_some() {
            self.update_notifier.notify_waiters();
        }

        Ok(())
    }

    fn get_update_notifier(&self) -> Arc<Notify> {
        self.update_notifier.clone()
    }

    async fn list_chat_template_heads(&self) -> Result<Vec<ChatTemplateHead>> {
        Ok(self
            .chat_templates
            .iter()
            .map(|entry| entry.value().to_head())
            .collect())
    }

    async fn read_agent_desired_state(&self) -> Result<AgentDesiredState> {
        Ok(self
            .agent_desired_state
            .read()
            .expect("Failed to acquire read lock")
            .clone())
    }

    async fn read_chat_template(&self, id: String) -> Result<Option<ChatTemplate>> {
        Ok(self.chat_templates.get(&id).map(|template| template.clone()))
    }

    async fn store_agent_desired_state(&self, state: &AgentDesiredState) -> Result<()> {
        {
            let mut agent_desired_state = self
                .agent_desired_state
                .write()
                .expect("Failed to acquire write lock");

            *agent_desired_state = state.clone();
        }

        self.update_notifier.notify_waiters();

        Ok(())
    }

    async fn store_chat_template(&self, chat_template: &ChatTemplate) -> Result<()> {
        self.chat_templates.insert(chat_template.id.clone(), chat_template.clone());
        self.update_notifier.notify_waiters();

        Ok(())
    }
}
