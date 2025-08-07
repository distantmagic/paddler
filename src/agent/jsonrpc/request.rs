use serde::Deserialize;
use serde::Serialize;

use crate::request_params::ContinueFromConversationHistoryParams;
use crate::request_params::ContinueFromRawPromptParams;
use crate::request_params::GenerateEmbeddingBatchParams;

#[derive(Deserialize, Serialize)]
pub enum Request {
    ContinueFromConversationHistory(ContinueFromConversationHistoryParams),
    ContinueFromRawPrompt(ContinueFromRawPromptParams),
    GenerateEmbeddingBatch(GenerateEmbeddingBatchParams),
    GetChatTemplateOverride,
    GetModelMetadata,
}

impl From<ContinueFromConversationHistoryParams> for Request {
    fn from(params: ContinueFromConversationHistoryParams) -> Self {
        Request::ContinueFromConversationHistory(params)
    }
}

impl From<ContinueFromRawPromptParams> for Request {
    fn from(params: ContinueFromRawPromptParams) -> Self {
        Request::ContinueFromRawPrompt(params)
    }
}

impl From<GenerateEmbeddingBatchParams> for Request {
    fn from(params: GenerateEmbeddingBatchParams) -> Self {
        Request::GenerateEmbeddingBatch(params)
    }
}
