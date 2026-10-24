use serde::Deserialize;
use serde::Serialize;

use crate::chat_template_conversation::ChatTemplateConversation;
use crate::conversation_message::ConversationMessage;
use crate::media_marker::MediaMarker;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ConversationHistory {
    pub messages: Vec<ConversationMessage>,
}

impl ConversationHistory {
    #[must_use]
    pub const fn new(messages: Vec<ConversationMessage>) -> Self {
        Self { messages }
    }

    #[must_use]
    pub fn into_chat_template_conversation(
        self,
        media_marker: &MediaMarker,
    ) -> ChatTemplateConversation {
        let mut image_urls = Vec::new();
        let messages = self
            .messages
            .into_iter()
            .map(|message| message.into_chat_template_message(media_marker, &mut image_urls))
            .collect();

        ChatTemplateConversation {
            image_urls,
            messages,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use serde_json::json;
    use serde_json::to_value;

    use super::ConversationHistory;
    use crate::chat_template_conversation::ChatTemplateConversation;
    use crate::chat_template_message::ChatTemplateMessage;
    use crate::chat_template_message_content::ChatTemplateMessageContent;
    use crate::chat_template_message_content_part::ChatTemplateMessageContentPart;
    use crate::conversation_message::ConversationMessage;
    use crate::conversation_message_content::ConversationMessageContent;
    use crate::conversation_message_content_part::ConversationMessageContentPart;
    use crate::conversation_tool_call::ConversationToolCall;
    use crate::image_url::ImageUrl;
    use crate::media_marker::MediaMarker;

    fn make_text_message(role: &str, text: &str) -> ConversationMessage {
        ConversationMessage::Message {
            content: ConversationMessageContent::Text(text.to_owned()),
            role: role.to_owned(),
        }
    }

    fn make_parts_message(
        role: &str,
        parts: Vec<ConversationMessageContentPart>,
    ) -> ConversationMessage {
        ConversationMessage::Message {
            content: ConversationMessageContent::Parts(parts),
            role: role.to_owned(),
        }
    }

    fn rendered_messages(history: ConversationHistory) -> Value {
        to_value(
            history
                .into_chat_template_conversation(&MediaMarker::new("[IMAGE]".to_owned()))
                .messages,
        )
        .unwrap()
    }

    #[test]
    fn collects_image_urls_and_replaces_images_with_the_media_marker() {
        let history = ConversationHistory::new(vec![
            make_text_message("assistant", "hello"),
            make_parts_message(
                "user",
                vec![
                    ConversationMessageContentPart::Text {
                        text: "before".to_owned(),
                    },
                    ConversationMessageContentPart::ImageUrl {
                        image_url: ImageUrl {
                            url: "http://example.com/img.png".to_owned(),
                        },
                    },
                    ConversationMessageContentPart::Text {
                        text: "after".to_owned(),
                    },
                ],
            ),
        ]);

        assert_eq!(
            history.into_chat_template_conversation(&MediaMarker::new("[IMAGE]".to_owned())),
            ChatTemplateConversation {
                image_urls: vec![ImageUrl {
                    url: "http://example.com/img.png".to_owned(),
                }],
                messages: vec![
                    ChatTemplateMessage::Message {
                        content: ChatTemplateMessageContent::Text("hello".to_owned()),
                        role: "assistant".to_owned(),
                    },
                    ChatTemplateMessage::Message {
                        content: ChatTemplateMessageContent::Parts(vec![
                            ChatTemplateMessageContentPart {
                                content_type: "text",
                                text: "before".to_owned(),
                            },
                            ChatTemplateMessageContentPart {
                                content_type: "text",
                                text: "[IMAGE]".to_owned(),
                            },
                            ChatTemplateMessageContentPart {
                                content_type: "text",
                                text: "after".to_owned(),
                            },
                        ]),
                        role: "user".to_owned(),
                    },
                ],
            }
        );
    }

    #[test]
    fn an_assistant_tool_call_turn_reaches_the_template_with_structured_tool_calls() {
        let history = ConversationHistory::new(vec![ConversationMessage::AssistantToolCalls {
            content: ConversationMessageContent::Text(String::new()),
            tool_calls: vec![ConversationToolCall {
                arguments: json!({ "location": "Paris" }).as_object().unwrap().clone(),
                id: "call_weather".to_owned(),
                name: "get_weather".to_owned(),
            }],
        }]);

        assert_eq!(
            rendered_messages(history),
            json!([{
                "content": "",
                "role": "assistant",
                "tool_calls": [{
                    "function": { "arguments": { "location": "Paris" }, "name": "get_weather" },
                    "id": "call_weather",
                    "type": "function"
                }]
            }])
        );
    }

    #[test]
    fn a_tool_result_reaches_the_template_with_its_tool_call_id() {
        let history = ConversationHistory::new(vec![ConversationMessage::ToolResult {
            content: ConversationMessageContent::Text("sunny".to_owned()),
            tool_call_id: "call_weather".to_owned(),
        }]);

        assert_eq!(
            rendered_messages(history),
            json!([{ "content": "sunny", "role": "tool", "tool_call_id": "call_weather" }])
        );
    }
}
