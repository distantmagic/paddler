use serde::Serialize;

use crate::chat_template_message_content::ChatTemplateMessageContent;
use crate::chat_template_tool_call::ChatTemplateToolCall;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ChatTemplateMessage {
    AssistantToolCalls {
        content: ChatTemplateMessageContent,
        role: &'static str,
        tool_calls: Vec<ChatTemplateToolCall>,
    },
    Message {
        content: ChatTemplateMessageContent,
        role: String,
    },
    ToolResult {
        content: ChatTemplateMessageContent,
        role: &'static str,
        tool_call_id: String,
    },
}
