use std::sync::Arc;

use log::error;

use crate::agent::generate_tokens_stopper_collection::GenerateTokensStopperCollection;

pub struct GenerateTokensStopperDropGuard {
    pub generate_tokens_stopper_collection: Arc<GenerateTokensStopperCollection>,
    pub request_id: String,
}

impl Drop for GenerateTokensStopperDropGuard {
    fn drop(&mut self) {
        if let Err(err) = self
            .generate_tokens_stopper_collection
            .deregister_stopper(self.request_id.clone())
        {
            error!(
                "Failed to deregister stopper for request_id {}: {}",
                self.request_id, err
            );
        }
    }
}
