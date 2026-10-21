use serde::Deserialize;
use serde::Serialize;

use crate::assistant_role::ASSISTANT_ROLE;
use crate::chat_template_message::ChatTemplateMessage;
use crate::conversation_message_content::ConversationMessageContent;
use crate::conversation_tool_call::ConversationToolCall;
use crate::image_url::ImageUrl;
use crate::media_marker::MediaMarker;
use crate::tool_role::TOOL_ROLE;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub enum ConversationMessage {
    AssistantToolCalls {
        content: ConversationMessageContent,
        tool_calls: Vec<ConversationToolCall>,
    },
    Message {
        content: ConversationMessageContent,
        role: String,
    },
    ToolResult {
        content: ConversationMessageContent,
        tool_call_id: String,
    },
}

impl ConversationMessage {
    pub fn into_chat_template_message(
        self,
        media_marker: &MediaMarker,
        image_urls: &mut Vec<ImageUrl>,
    ) -> ChatTemplateMessage {
        match self {
            Self::AssistantToolCalls {
                content,
                tool_calls,
            } => ChatTemplateMessage::AssistantToolCalls {
                content: content.into_chat_template_content(media_marker, image_urls),
                role: ASSISTANT_ROLE,
                tool_calls: tool_calls.into_iter().map(Into::into).collect(),
            },
            Self::Message { content, role } => ChatTemplateMessage::Message {
                content: content.into_chat_template_content(media_marker, image_urls),
                role,
            },
            Self::ToolResult {
                content,
                tool_call_id,
            } => ChatTemplateMessage::ToolResult {
                content: content.into_chat_template_content(media_marker, image_urls),
                role: TOOL_ROLE,
                tool_call_id,
            },
        }
    }
}
