use serde::Deserialize;
use serde::Serialize;

use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::model_metadata::ModelMetadata;

#[derive(Deserialize, Serialize)]
pub enum Response {
    GeneratedToken(GeneratedTokenEnvelope),
    ModelMetadata(Option<ModelMetadata>),
}
