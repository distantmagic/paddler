use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use actix_web::Error;
use actix_web::Responder;
use actix_web::post;
use actix_web::web;
use async_trait::async_trait;
use nanoid::nanoid;
use serde::Deserialize;
use serde_json::json;

use crate::balancer::chunk_forwarding_session_controller::transforms_outgoing_message::TransformsOutgoingMessage;
use crate::balancer::compatibility::openai_service::app_data::AppData;
use crate::balancer::inference_client::Message as OutgoingMessage;
use crate::balancer::inference_client::Response as OutgoingResponse;
use crate::balancer::stream_from_agent::stream_from_agent;
use crate::conversation_message::ConversationMessage;
use crate::generated_token_result::GeneratedTokenResult;
use crate::jsonrpc::ResponseEnvelope;
use crate::request_params::ContinueFromConversationHistoryParams;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

#[derive(Deserialize)]
/// Although fields are same as in Paddler's conversation message for the moment,
/// it would be better if this struct stayed independent from ours just in case
/// to avoid any potential side effects in the future.
struct OpenAIMessage {
    content: String,
    role: String,
}

impl OpenAIMessage {
    fn to_paddler_message(&self) -> ConversationMessage {
        ConversationMessage {
            content: self.content.clone(),
            role: self.role.clone(),
        }
    }
}

#[derive(Deserialize)]
struct OpenAICompletionRequestParams {
    max_completion_tokens: Option<i32>,
    messages: Vec<OpenAIMessage>,
    /// This parameter is ignored here, but is required by the OpenAI API.
    model: String,
    stream: bool,
}

#[derive(Clone)]
struct OpenAIResponseTransformer {
    model: String,
}

#[async_trait]
impl TransformsOutgoingMessage for OpenAIResponseTransformer {
    type TransformedMessage = serde_json::Value;

    async fn transform(&self, message: OutgoingMessage) -> anyhow::Result<serde_json::Value> {
        if let OutgoingMessage::Response(ResponseEnvelope {
            request_id,
            response: OutgoingResponse::GeneratedToken(GeneratedTokenResult::Token(token)),
        }) = message
        {
            Ok(json!({
                "id": request_id,
                "object": "chat.completion.chunk",
                "created": current_timestamp(),
                "model": self.model,
                "system_fingerprint": nanoid!(),
                "choices": [
                    {
                        "index": 0,
                        "delta": {
                            "role": "assistant",
                            "content": token,
                        },
                        "logprobs": null,
                        "finish_reason": null
                    }
                ]
            }))
        } else {
            Ok(serde_json::to_value(&message)?)
        }
    }
}

#[post("/v1/chat_completions")]
async fn respond(
    app_data: web::Data<AppData>,
    params: web::Json<OpenAICompletionRequestParams>,
) -> Result<impl Responder, Error> {
    stream_from_agent(
        app_data.buffered_request_manager.clone(),
        app_data.inference_service_configuration.clone(),
        ContinueFromConversationHistoryParams {
            add_generation_prompt: true,
            conversation_history: params
                .messages
                .iter()
                .map(|openai_message| openai_message.to_paddler_message())
                .collect(),
            enable_thinking: true,
            max_tokens: params.max_completion_tokens.unwrap_or(2000),
            tools: vec![],
        },
        OpenAIResponseTransformer {
            model: params.model.clone(),
        },
    )
    .await
}
