use serde::Deserialize;
use serde::Serialize;

use crate::chat_template::ChatTemplate;
use crate::embedding_result::EmbeddingResult;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::model_metadata::ModelMetadata;

#[derive(Deserialize, Serialize)]
pub enum Response {
    ChatTemplateOverride(Option<ChatTemplate>),
    Embedding(EmbeddingResult),
    GeneratedToken(GeneratedTokenEnvelope),
    ModelMetadata(Option<ModelMetadata>),
}

impl From<Option<ChatTemplate>> for Response {
    fn from(chat_template: Option<ChatTemplate>) -> Self {
        Response::ChatTemplateOverride(chat_template)
    }
}

impl From<EmbeddingResult> for Response {
    fn from(embedding_result: EmbeddingResult) -> Self {
        Response::Embedding(embedding_result)
    }
}

impl From<GeneratedTokenEnvelope> for Response {
    fn from(generated_token_envelope: GeneratedTokenEnvelope) -> Self {
        Response::GeneratedToken(generated_token_envelope)
    }
}

impl From<Option<ModelMetadata>> for Response {
    fn from(model_metadata: Option<ModelMetadata>) -> Self {
        Response::ModelMetadata(model_metadata)
    }
}
