use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::balancer::generate_tokens_forward_result::GenerateTokensForwardResult;
use crate::response::ChunkResponse;

pub struct GenerateTokensSenderCollection {
    senders: DashMap<String, mpsc::Sender<ChunkResponse>>,
}

impl GenerateTokensSenderCollection {
    pub fn new() -> Self {
        GenerateTokensSenderCollection {
            senders: DashMap::new(),
        }
    }

    pub fn deregister_sender(&self, request_id: String) {
        self.senders.remove(&request_id);
    }

    pub async fn forward_response(
        &self,
        request_id: String,
        response: ChunkResponse,
    ) -> Result<GenerateTokensForwardResult> {
        if let Some(sender) = self.senders.get(&request_id) {
            sender.send(response).await?;

            Ok(GenerateTokensForwardResult::Forwarded)
        } else {
            Ok(GenerateTokensForwardResult::NoSenderFound(request_id))
        }
    }

    pub fn register_sender(&self, request_id: String, sender: mpsc::Sender<ChunkResponse>) {
        self.senders.insert(request_id, sender);
    }
}
