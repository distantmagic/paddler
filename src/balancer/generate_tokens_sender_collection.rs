use anyhow::anyhow;
use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::generated_token_envelope::GeneratedTokenEnvelope;

pub struct GenerateTokensSenderCollection {
    generate_tokens_senders: DashMap<String, mpsc::UnboundedSender<GeneratedTokenEnvelope>>,
}

impl GenerateTokensSenderCollection {
    pub fn new() -> Self {
        Self {
            generate_tokens_senders: DashMap::new(),
        }
    }

    pub fn deregister_sender(&self, request_id: String) -> Result<()> {
        if let Some(sender) = self.generate_tokens_senders.remove(&request_id) {
            drop(sender);

            Ok(())
        } else {
            Err(anyhow!("No sender found for request_id {request_id}"))
        }
    }

    pub async fn forward_generated_token(
        &self,
        request_id: String,
        generated_token_envelope: GeneratedTokenEnvelope,
    ) -> Result<()> {
        if let Some(sender) = self.generate_tokens_senders.get(&request_id) {
            sender.send(generated_token_envelope)?;

            Ok(())
        } else {
            Err(anyhow!("No sender found for request_id {request_id}"))
        }
    }

    pub fn register_sender(
        &self,
        request_id: String,
        sender: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
    ) -> Result<()> {
        if self.generate_tokens_senders.contains_key(&request_id) {
            return Err(anyhow!("Sender for request_id {request_id} already exists"));
        }

        self.generate_tokens_senders.insert(request_id, sender);

        Ok(())
    }
}
