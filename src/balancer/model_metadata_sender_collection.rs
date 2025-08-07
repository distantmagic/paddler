use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::balancer::manages_senders::ManagesSenders;
use crate::model_metadata::ModelMetadata;

pub struct ModelMetadataSenderCollection {
    senders: DashMap<String, mpsc::UnboundedSender<Option<ModelMetadata>>>,
}

impl ModelMetadataSenderCollection {
    pub fn new() -> Self {
        Self {
            senders: DashMap::new(),
        }
    }
}

#[async_trait]
impl ManagesSenders for ModelMetadataSenderCollection {
    type Value = Option<ModelMetadata>;

    fn get_sender_collection(&self) -> &DashMap<String, mpsc::UnboundedSender<Self::Value>> {
        &self.senders
    }
}
