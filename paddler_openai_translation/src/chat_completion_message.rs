use serde::Deserialize;

use paddler_messaging::assistant_role::ASSISTANT_ROLE;
use paddler_messaging::conversation_message::ConversationMessage;
use paddler_messaging::conversation_message_content::ConversationMessageContent;
use paddler_messaging::tool_role::TOOL_ROLE;

use crate::chat_completion_requested_tool_call::ChatCompletionRequestedToolCall;
use crate::openai_translation_error::OpenAITranslationError;

#[derive(Deserialize)]
pub struct ChatCompletionMessage {
    pub content: Option<ConversationMessageContent>,
    pub role: String,
    pub tool_call_id: Option<String>,
    pub tool_calls: Option<Vec<ChatCompletionRequestedToolCall>>,
}

impl ChatCompletionMessage {
    pub fn into_conversation_message(self) -> Result<ConversationMessage, OpenAITranslationError> {
        let Self {
            content,
            role,
            tool_call_id,
            tool_calls,
        } = self;

        match tool_calls {
            Some(tool_calls) if !tool_calls.is_empty() => {
                if role != ASSISTANT_ROLE {
                    return Err(OpenAITranslationError::ToolCallsOnNonAssistantMessage { role });
                }

                Ok(ConversationMessage::AssistantToolCalls {
                    content: content
                        .unwrap_or_else(|| ConversationMessageContent::Text(String::new())),
                    tool_calls: tool_calls
                        .into_iter()
                        .map(ChatCompletionRequestedToolCall::into_conversation_tool_call)
                        .collect::<Result<_, _>>()?,
                })
            }
            Some(_) | None => {
                let Some(content) = content else {
                    return Err(OpenAITranslationError::MessageWithoutContent { role });
                };

                if role != TOOL_ROLE {
                    return Ok(ConversationMessage::Message { content, role });
                }

                tool_call_id
                    .map(|tool_call_id| ConversationMessage::ToolResult {
                        content,
                        tool_call_id,
                    })
                    .ok_or(OpenAITranslationError::ToolResultWithoutToolCallId)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use serde_json::from_value;
    use serde_json::json;

    use paddler_messaging::conversation_message::ConversationMessage;
    use paddler_messaging::conversation_message_content::ConversationMessageContent;
    use paddler_messaging::conversation_message_content_part::ConversationMessageContentPart;
    use paddler_messaging::conversation_tool_call::ConversationToolCall;
    use paddler_messaging::image_url::ImageUrl;

    use super::ChatCompletionMessage;
    use crate::openai_translation_error::OpenAITranslationError;

    fn converted(message: Value) -> Result<ConversationMessage, OpenAITranslationError> {
        from_value::<ChatCompletionMessage>(message)
            .unwrap()
            .into_conversation_message()
    }

    fn read_call(id: &str, path: &str) -> ConversationToolCall {
        ConversationToolCall {
            arguments: json!({ "path": path }).as_object().unwrap().clone(),
            id: id.to_owned(),
            name: "read".to_owned(),
        }
    }

    #[test]
    fn a_user_message_with_parts_keeps_its_role_and_parts() {
        assert_eq!(
            converted(json!({
                "role": "user",
                "content": [
                    {"type": "text", "text": "OCR this"},
                    {"type": "image_url", "image_url": {"url": "data:image/png;base64,abc"}}
                ]
            }))
            .unwrap(),
            ConversationMessage::Message {
                content: ConversationMessageContent::Parts(vec![
                    ConversationMessageContentPart::Text {
                        text: "OCR this".to_owned(),
                    },
                    ConversationMessageContentPart::ImageUrl {
                        image_url: ImageUrl {
                            url: "data:image/png;base64,abc".to_owned(),
                        },
                    },
                ]),
                role: "user".to_owned(),
            }
        );
    }

    #[test]
    fn parallel_tool_calls_without_content_become_one_assistant_turn_with_empty_text() {
        assert_eq!(
            converted(json!({
                "role": "assistant",
                "content": null,
                "tool_calls": [
                    {"id": "call_0", "type": "function", "function": {"name": "read", "arguments": "{\"path\":\"a.txt\"}"}},
                    {"id": "call_1", "type": "function", "function": {"name": "read", "arguments": "{\"path\":\"b.txt\"}"}}
                ]
            }))
            .unwrap(),
            ConversationMessage::AssistantToolCalls {
                content: ConversationMessageContent::Text(String::new()),
                tool_calls: vec![read_call("call_0", "a.txt"), read_call("call_1", "b.txt")],
            }
        );
    }

    #[test]
    fn assistant_text_alongside_tool_calls_stays_in_the_same_turn() {
        assert_eq!(
            converted(json!({
                "role": "assistant",
                "content": "Let me read it.",
                "tool_calls": [
                    {"id": "call_0", "type": "function", "function": {"name": "read", "arguments": "{\"path\":\"a.txt\"}"}}
                ]
            }))
            .unwrap(),
            ConversationMessage::AssistantToolCalls {
                content: ConversationMessageContent::Text("Let me read it.".to_owned()),
                tool_calls: vec![read_call("call_0", "a.txt")],
            }
        );
    }

    #[test]
    fn a_tool_message_keeps_its_tool_call_id() {
        assert_eq!(
            converted(
                json!({"role": "tool", "tool_call_id": "call_0", "content": "file contents"})
            )
            .unwrap(),
            ConversationMessage::ToolResult {
                content: ConversationMessageContent::Text("file contents".to_owned()),
                tool_call_id: "call_0".to_owned(),
            }
        );
    }

    #[test]
    fn a_message_without_content_or_tool_calls_is_refused_naming_its_role() {
        assert!(matches!(
            converted(json!({"role": "assistant", "content": null})),
            Err(OpenAITranslationError::MessageWithoutContent { role }) if role == "assistant"
        ));
    }

    #[test]
    fn tool_calls_on_a_message_that_is_not_an_assistant_message_are_refused() {
        assert!(matches!(
            converted(json!({
                "role": "user",
                "content": "hi",
                "tool_calls": [
                    {"id": "call_0", "type": "function", "function": {"name": "read", "arguments": "{}"}}
                ]
            })),
            Err(OpenAITranslationError::ToolCallsOnNonAssistantMessage { role }) if role == "user"
        ));
    }

    #[test]
    fn a_tool_message_without_a_tool_call_id_is_refused() {
        assert!(matches!(
            converted(json!({"role": "tool", "content": "file contents"})),
            Err(OpenAITranslationError::ToolResultWithoutToolCallId)
        ));
    }

    #[test]
    fn tool_call_arguments_that_are_not_a_json_object_are_refused() {
        assert!(matches!(
            converted(json!({
                "role": "assistant",
                "content": null,
                "tool_calls": [
                    {"id": "call_0", "type": "function", "function": {"name": "read", "arguments": "[\"a.txt\"]"}}
                ]
            })),
            Err(OpenAITranslationError::ToolCallArgumentsAreNotAJsonObject { tool_call_id, .. })
                if tool_call_id == "call_0"
        ));
    }
}
