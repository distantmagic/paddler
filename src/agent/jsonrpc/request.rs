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
