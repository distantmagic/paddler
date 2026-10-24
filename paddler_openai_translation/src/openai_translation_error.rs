use std::time::SystemTimeError;

use serde_json::Error as SerdeJsonError;

use paddler_messaging::request_params_validation_error::RequestParamsValidationError;

#[derive(Debug, thiserror::Error)]
pub enum OpenAITranslationError {
    #[error("the system clock reads a time before the Unix epoch")]
    ClockBeforeUnixEpoch(#[source] SystemTimeError),

    #[error("a '{role}' message must have content")]
    MessageWithoutContent { role: String },

    #[error("the arguments of the tool call '{tool_call_id}' are not a JSON object")]
    ToolCallArgumentsAreNotAJsonObject {
        tool_call_id: String,
        #[source]
        source: SerdeJsonError,
    },

    #[error("only an assistant message can request tool calls, not a '{role}' message")]
    ToolCallsOnNonAssistantMessage { role: String },

    #[error(transparent)]
    ToolRejected(#[from] RequestParamsValidationError),

    #[error("a tool message must name the tool call it answers in 'tool_call_id'")]
    ToolResultWithoutToolCallId,
}
