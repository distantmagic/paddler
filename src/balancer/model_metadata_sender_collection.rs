use anyhow::anyhow;
use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::model_metadata::ModelMetadata;

pub struct ModelMetadataSenderCollection {
    model_metadata_senders: DashMap<String, mpsc::UnboundedSender<Option<ModelMetadata>>>,
}

impl ModelMetadataSenderCollection {
    pub fn new() -> Self {
        Self {
            model_metadata_senders: DashMap::new(),
        }
    }

    pub fn deregister_sender(&self, request_id: String) -> Result<()> {
        if let Some(sender) = self.model_metadata_senders.remove(&request_id) {
            drop(sender);

            Ok(())
        } else {
            Err(anyhow!("No sender found for request_id {request_id}"))
        }
    }

    pub async fn forward_model_metadata(
        &self,
        request_id: String,
        model_metadata: Option<ModelMetadata>,
    ) -> Result<()> {
        if let Some(sender) = self.model_metadata_senders.get(&request_id) {
            sender.send(model_metadata)?;

            Ok(())
        } else {
            Err(anyhow!("No sender found for request_id {request_id}"))
        }
    }

    pub fn register_sender(
        &self,
        request_id: String,
        sender: mpsc::UnboundedSender<Option<ModelMetadata>>,
    ) -> Result<()> {
        if self.model_metadata_senders.contains_key(&request_id) {
            return Err(anyhow!("Sender for request_id {request_id} already exists"));
        }

        self.model_metadata_senders.insert(request_id, sender);

        Ok(())
    }
}
