use serde_json::json;

use paddler_messaging::conversation_message::ConversationMessage;
use paddler_messaging::conversation_message_content::ConversationMessageContent;

use crate::assistant_role::ASSISTANT_ROLE;

#[must_use]
pub fn tool_call_conversation_message(
    call_id: &str,
    name: &str,
    arguments: &str,
) -> ConversationMessage {
    ConversationMessage {
        content: ConversationMessageContent::Text(
            json!({ "call_id": call_id, "name": name, "arguments": arguments }).to_string(),
        ),
        role: ASSISTANT_ROLE.to_owned(),
    }
}
