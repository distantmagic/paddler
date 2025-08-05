use serde::Deserialize;
use serde::Serialize;

use crate::conversation_message::ConversationMessage;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ContinueFromConversationHistoryParams {
    pub add_generation_prompt: bool,
    pub conversation_history: Vec<ConversationMessage>,
    pub enable_thinking: bool,
    pub max_tokens: i32,
}
