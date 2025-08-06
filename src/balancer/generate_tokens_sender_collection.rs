use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::balancer::manages_senders::ManagesSenders;
use crate::generated_token_envelope::GeneratedTokenEnvelope;

pub struct GenerateTokensSenderCollection {
    senders: DashMap<String, mpsc::UnboundedSender<GeneratedTokenEnvelope>>,
}

impl GenerateTokensSenderCollection {
    pub fn new() -> Self {
        Self {
            senders: DashMap::new(),
        }
    }
}

#[async_trait]
impl ManagesSenders for GenerateTokensSenderCollection {
    type Value = GeneratedTokenEnvelope;

    fn get_sender_collection(&self) -> &DashMap<String, mpsc::UnboundedSender<Self::Value>> {
        &self.senders
    }
}
