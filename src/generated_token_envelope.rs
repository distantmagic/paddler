use serde::Deserialize;
use serde::Serialize;

use crate::generated_token_result::GeneratedTokenResult;

#[derive(Debug, Deserialize, Serialize)]
pub struct GeneratedTokenEnvelope {
    pub generated_token_result: GeneratedTokenResult,
    pub slot: u32,
}
