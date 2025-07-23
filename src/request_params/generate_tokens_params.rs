use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GenerateTokensParams {
    pub max_tokens: i32,
    pub prompt: String,
}
