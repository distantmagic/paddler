use actix_web::web::Data;

use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;

pub struct GenerateTokensSenderGuard {
    pub request_id: String,
    pub sender_collection: Data<GenerateTokensSenderCollection>,
}

impl Drop for GenerateTokensSenderGuard {
    fn drop(&mut self) {
        self.sender_collection
            .deregister_sender(self.request_id.clone());
    }
}
