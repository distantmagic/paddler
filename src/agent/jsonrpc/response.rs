use serde::Deserialize;
use serde::Serialize;

use crate::generated_token_envelope::GeneratedTokenEnvelope;

#[derive(Deserialize, Serialize)]
pub enum Response {
    GeneratedToken(GeneratedTokenEnvelope),
}
