use serde::Deserialize;

use paddler_messaging::conversation_message::ConversationMessage;

use crate::chat_completion_requested_function::ChatCompletionRequestedFunction;
use crate::tool_call_conversation_message::tool_call_conversation_message;

#[derive(Deserialize)]
pub struct ChatCompletionRequestedToolCall {
    pub function: ChatCompletionRequestedFunction,
    pub id: String,
}

impl ChatCompletionRequestedToolCall {
    #[must_use]
    pub fn into_conversation_message(self) -> ConversationMessage {
        tool_call_conversation_message(&self.id, &self.function.name, &self.function.arguments)
    }
}
