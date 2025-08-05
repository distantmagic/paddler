use serde::Deserialize;
use serde::Serialize;

use crate::request_params::ContinueFromConversationHistoryParams;
use crate::request_params::ContinueFromRawPromptParams;

#[derive(Deserialize, Serialize)]
pub enum Request {
    ContinueFromConversationHistory(ContinueFromConversationHistoryParams),
    ContinueFromRawPrompt(ContinueFromRawPromptParams),
}
