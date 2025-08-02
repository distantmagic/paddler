use serde::Deserialize;
use serde::Serialize;

use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::model_metadata::ModelMetadata;
use crate::chat_template::ChatTemplate;

#[derive(Deserialize, Serialize)]
pub enum Response {
    ChatTemplateOverride(Option<ChatTemplate>),
    GeneratedToken(GeneratedTokenEnvelope),
    ModelMetadata(Option<ModelMetadata>),
}
