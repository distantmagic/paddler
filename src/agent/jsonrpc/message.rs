use serde::Deserialize;
use serde::Serialize;

use super::Notification;
use super::Request;
use crate::jsonrpc::Error;
use crate::jsonrpc::ErrorEnvelope;
use crate::jsonrpc::RequestEnvelope;
use crate::rpc_message::RpcMessage;

#[derive(Deserialize, Serialize)]
pub enum Message {
    Error(ErrorEnvelope<Error>),
    Notification(Notification),
    Request(RequestEnvelope<Request>),
}

impl RpcMessage for Message {}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::super::notification_params::SetStateParams;
    use super::*;
    use crate::agent_desired_model::AgentDesiredModel;
    use crate::agent_desired_state::AgentDesiredState;
    use crate::huggingface_model_reference::HuggingFaceModelReference;

    #[test]
    fn test_message_serialization() -> Result<()> {
        let serialized = serde_json::to_string(&Message::Notification(Notification::SetState(
            SetStateParams {
                desired_state: AgentDesiredState {
                    model: AgentDesiredModel::HuggingFace(HuggingFaceModelReference {
                        filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
                        repo_id: "Qwen/Qwen3-0.6B-GGUF".to_string(),
                        revision: "main".to_string(),
                    }),
                },
            },
        )))?;

        assert_eq!(
            serialized,
            r#"{"Notification":{"SetState":{"desired_state":{"model":{"HuggingFace":{"branch":"main","filename":"Qwen3-0.6B-Q8_0.gguf","repo":"Qwen/Qwen3-0.6B-GGUF"}}}}}}"#
        );

        Ok(())
    }
}
