use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestEnvelope<TRequest> {
    pub id: String,
    pub request: TRequest,
}
