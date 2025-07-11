use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct GenerateTokens {
    pub prompt: String,
}
