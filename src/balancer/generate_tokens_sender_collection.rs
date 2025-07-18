use anyhow::anyhow;
use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::mpsc;

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
    ) -> Result<()> {
        if let Some(sender) = self.senders.get(&request_id) {
            sender.send(response).await?;
        } else {
            return Err(anyhow!("No sender found for request ID: {}", request_id));
        }

        Ok(())
    }

    pub fn register_sender(&self, request_id: String, sender: mpsc::Sender<ChunkResponse>) {
        self.senders.insert(request_id, sender);
    }
}
