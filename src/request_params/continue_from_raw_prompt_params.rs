use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContinueFromRawPromptParams {
    pub max_tokens: i32,
    pub raw_prompt: String,
}
