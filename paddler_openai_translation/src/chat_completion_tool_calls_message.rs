use serde::Deserialize;

use paddler_messaging::conversation_message::ConversationMessage;
use paddler_messaging::conversation_message_content::ConversationMessageContent;

use crate::chat_completion_requested_tool_call::ChatCompletionRequestedToolCall;

#[derive(Deserialize)]
pub struct ChatCompletionToolCallsMessage {
    pub content: Option<ConversationMessageContent>,
    pub role: String,
    pub tool_calls: Vec<ChatCompletionRequestedToolCall>,
}

impl ChatCompletionToolCallsMessage {
    #[must_use]
    pub fn into_conversation_messages(self) -> Vec<ConversationMessage> {
        let Self {
            content,
            role,
            tool_calls,
        } = self;

        content
            .map(|content| ConversationMessage { content, role })
            .into_iter()
            .chain(
                tool_calls
                    .into_iter()
                    .map(ChatCompletionRequestedToolCall::into_conversation_message),
            )
            .collect()
    }
}
