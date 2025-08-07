use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseEnvelope<TResponse> {
    pub request_id: String,
    pub response: TResponse,
}
