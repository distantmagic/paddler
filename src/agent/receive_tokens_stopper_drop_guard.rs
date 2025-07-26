use std::sync::Arc;

use log::error;

use crate::agent::receive_tokens_stopper_collection::ReceiveTokensStopperCollection;

pub struct ReceiveTokensStopperDropGuard {
    pub receive_tokens_stopper_collection: Arc<ReceiveTokensStopperCollection>,
    pub request_id: String,
}

impl Drop for ReceiveTokensStopperDropGuard {
    fn drop(&mut self) {
        if let Err(err) = self
            .receive_tokens_stopper_collection
            .deregister_stopper(self.request_id.clone())
        {
            error!(
                "Failed to deregister stopper for request_id {}: {}",
                self.request_id, err
            );
        }
    }
}
