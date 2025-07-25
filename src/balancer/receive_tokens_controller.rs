use actix_web::web::Data;
use log::error;
use tokio::sync::mpsc;

use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::generated_token_envelope::GeneratedTokenEnvelope;

pub struct ReceiveTokensController {
    pub generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    pub generated_tokens_rx: mpsc::UnboundedReceiver<GeneratedTokenEnvelope>,
    pub request_id: String,
}

impl Drop for ReceiveTokensController {
    fn drop(&mut self) {
        self.generate_tokens_sender_collection
            .deregister_sender(self.request_id.clone())
            .unwrap_or_else(|err| {
                error!(
                    "Failed to deregister sender for request_id {}: {err}",
                    self.request_id
                );
            });
    }
}
