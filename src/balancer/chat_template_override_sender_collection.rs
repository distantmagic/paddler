use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::chat_template::ChatTemplate;
use crate::balancer::manages_senders::ManagesSenders;

pub struct ChatTemplateOverrideSenderCollection {
    senders: DashMap<String, mpsc::UnboundedSender<Option<ChatTemplate>>>,
}

impl ChatTemplateOverrideSenderCollection {
    pub fn new() -> Self {
        Self {
            senders: DashMap::new(),
        }
    }
}

#[async_trait]
impl ManagesSenders for ChatTemplateOverrideSenderCollection {
    type Value = Option<ChatTemplate>;

    fn get_sender_collection(&self) -> &DashMap<String, mpsc::UnboundedSender<Self::Value>> {
        &self.senders
    }
}
