use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct RequestEnvelope<TRequest> {
    pub id: String,
    pub request: TRequest,
}
