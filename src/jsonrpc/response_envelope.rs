use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct ResponseEnvelope<TResponse> {
    pub request_id: String,
    pub response: TResponse,
}
