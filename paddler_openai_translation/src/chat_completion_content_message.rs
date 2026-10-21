use serde::Deserialize;

use paddler_messaging::conversation_message::ConversationMessage;
use paddler_messaging::conversation_message_content::ConversationMessageContent;

#[derive(Deserialize)]
pub struct ChatCompletionContentMessage {
    pub content: ConversationMessageContent,
    pub role: String,
}

impl ChatCompletionContentMessage {
    #[must_use]
    pub fn into_conversation_message(self) -> ConversationMessage {
        ConversationMessage {
            content: self.content,
            role: self.role,
        }
    }
}
