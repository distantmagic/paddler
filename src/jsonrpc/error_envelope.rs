use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorEnvelope<TRequest> {
    pub request_id: String,
    pub error: TRequest,
}
