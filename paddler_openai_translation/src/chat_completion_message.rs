use serde::Deserialize;

use paddler_messaging::conversation_message::ConversationMessage;

use crate::chat_completion_content_message::ChatCompletionContentMessage;
use crate::chat_completion_tool_calls_message::ChatCompletionToolCallsMessage;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ChatCompletionMessage {
    ToolCalls(ChatCompletionToolCallsMessage),
    Content(ChatCompletionContentMessage),
}

impl ChatCompletionMessage {
    #[must_use]
    pub fn into_conversation_messages(self) -> Vec<ConversationMessage> {
        match self {
            Self::ToolCalls(tool_calls_message) => tool_calls_message.into_conversation_messages(),
            Self::Content(content_message) => vec![content_message.into_conversation_message()],
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::from_value;
    use serde_json::json;

    use paddler_messaging::conversation_message::ConversationMessage;
    use paddler_messaging::conversation_message_content::ConversationMessageContent;

    use super::ChatCompletionMessage;
    use crate::tool_call_conversation_message::tool_call_conversation_message;

    #[test]
    fn openai_message_converts_to_conversation_message() {
        let input = json!({
            "role": "user",
            "content": [
                {"type": "text", "text": "OCR this"},
                {"type": "image_url", "image_url": {"url": "data:image/png;base64,abc"}}
            ]
        });

        let openai_message: ChatCompletionMessage = from_value(input).unwrap();
        let conversation_messages = openai_message.into_conversation_messages();

        assert_eq!(conversation_messages.len(), 1);
        assert_eq!(conversation_messages[0].role, "user");
        assert_eq!(conversation_messages[0].content.text_content(), "OCR this");
        assert_eq!(conversation_messages[0].content.image_urls().len(), 1);
    }

    #[test]
    fn an_assistant_tool_call_turn_without_content_becomes_one_message_per_call() {
        let openai_message: ChatCompletionMessage = from_value(json!({
            "role": "assistant",
            "content": null,
            "tool_calls": [
                {"id": "call_0", "type": "function", "function": {"name": "read", "arguments": "{\"path\":\"a.txt\"}"}},
                {"id": "call_1", "type": "function", "function": {"name": "read", "arguments": "{\"path\":\"b.txt\"}"}}
            ]
        }))
        .unwrap();

        assert_eq!(
            openai_message.into_conversation_messages(),
            vec![
                tool_call_conversation_message("call_0", "read", "{\"path\":\"a.txt\"}"),
                tool_call_conversation_message("call_1", "read", "{\"path\":\"b.txt\"}"),
            ]
        );
    }

    #[test]
    fn assistant_text_alongside_tool_calls_precedes_the_calls() {
        let openai_message: ChatCompletionMessage = from_value(json!({
            "role": "assistant",
            "content": "Let me read it.",
            "tool_calls": [
                {"id": "call_0", "type": "function", "function": {"name": "read", "arguments": "{}"}}
            ]
        }))
        .unwrap();

        assert_eq!(
            openai_message.into_conversation_messages(),
            vec![
                ConversationMessage {
                    content: ConversationMessageContent::Text("Let me read it.".to_owned()),
                    role: "assistant".to_owned(),
                },
                tool_call_conversation_message("call_0", "read", "{}"),
            ]
        );
    }
}
