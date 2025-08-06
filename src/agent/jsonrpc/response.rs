use serde::Deserialize;
use serde::Serialize;

use crate::chat_template::ChatTemplate;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::model_metadata::ModelMetadata;

#[derive(Deserialize, Serialize)]
pub enum Response {
    ChatTemplateOverride(Option<ChatTemplate>),
    GeneratedToken(GeneratedTokenEnvelope),
    ModelMetadata(Option<ModelMetadata>),
}
