use serde::Deserialize;
use serde::Serialize;

use crate::request_params::ContinueConversationParams;
use crate::request_params::GenerateTokensParams;

#[derive(Deserialize, Serialize)]
pub enum Request {
    ContinueConversation(ContinueConversationParams),
    GenerateTokens(GenerateTokensParams),
    GetModelMetadata,
}
