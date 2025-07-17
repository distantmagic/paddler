use serde::Deserialize;
use serde::Serialize;

use super::Notification;
use super::Request;
use crate::jsonrpc::Error;
use crate::jsonrpc::RequestEnvelope;
use crate::rpc_message::RpcMessage;

#[derive(Deserialize, Serialize)]
pub enum Message {
    Error(Error),
    Notification(Notification),
    Request(RequestEnvelope<Request>),
}

impl RpcMessage for Message {}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::super::notification_params::SetStateParams;
    use super::*;
    use crate::agent::huggingface_model_reference::HuggingFaceModelReference;
    use crate::agent::llamacpp_desired_model::LlamaCppDesiredModel;
    use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;

    #[test]
    fn test_message_serialization() -> Result<()> {
        let serialized = serde_json::to_string(&Message::Notification(Notification::SetState(
            SetStateParams {
                desired_state: LlamaCppDesiredState {
                    model: LlamaCppDesiredModel::HuggingFace(HuggingFaceModelReference {
                        filename: "Qwen3-0.6B-Q8_0.gguf".to_string(),
                        repo: "Qwen/Qwen3-0.6B-GGUF".to_string(),
                    }),
                },
            },
        )))?;

        assert_eq!(
            serialized,
            r#"{"Notification":{"SetState":{"desired_state":{"model":{"HuggingFace":{"filename":"Qwen3-0.6B-Q8_0.gguf","repo":"Qwen/Qwen3-0.6B-GGUF"}}}}}}"#
        );

        Ok(())
    }
}
