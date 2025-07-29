use actix_web::web::Data;
use log::error;
use tokio::sync::mpsc;

use crate::balancer::model_metadata_sender_collection::ModelMetadataSenderCollection;
use crate::model_metadata::ModelMetadata;

pub struct ReceiveModelMetadataController {
    pub model_metadata_rx: mpsc::UnboundedReceiver<Option<ModelMetadata>>,
    pub model_metadata_sender_collection: Data<ModelMetadataSenderCollection>,
    pub request_id: String,
}

impl Drop for ReceiveModelMetadataController {
    fn drop(&mut self) {
        self.model_metadata_sender_collection
            .deregister_sender(self.request_id.clone())
            .unwrap_or_else(|err| {
                error!(
                    "Failed to deregister sender for request_id {}: {err}",
                    self.request_id
                );
            });
    }
}
