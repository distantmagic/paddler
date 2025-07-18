pub mod jsonrpc;

use std::sync::Arc;

use actix_web::get;
use actix_web::rt;
use actix_web::web::Data;
use actix_web::web::Payload;
use actix_web::web::ServiceConfig;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_ws::Session;
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use self::jsonrpc::Message as InferenceJsonRpcMessage;
use self::jsonrpc::Request as InferenceJsonRpcRequest;
use crate::agent::jsonrpc::Message as AgentJsonRpcMessage;
use crate::agent::jsonrpc::Request as AgentJsonRpcRequest;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::generate_tokens_sender_guard::GenerateTokensSenderGuard;
use crate::controls_websocket_endpoint::ContinuationDecision;
use crate::controls_websocket_endpoint::ControlsWebSocketEndpoint;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::RequestEnvelope;
use crate::response::ChunkResponse;
use crate::sends_rpc_message::SendsRpcMessage as _;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

struct InferenceSocketControllerContext {
    agent_controller_pool: Data<AgentControllerPool>,
    generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
}

struct InferenceSocketController {
    agent_controller_pool: Data<AgentControllerPool>,
    generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
}

#[async_trait]
impl ControlsWebSocketEndpoint for InferenceSocketController {
    type Context = InferenceSocketControllerContext;
    type Message = InferenceJsonRpcMessage;

    fn create_context(&self) -> Self::Context {
        InferenceSocketControllerContext {
            agent_controller_pool: self.agent_controller_pool.clone(),
            generate_tokens_sender_collection: self.generate_tokens_sender_collection.clone(),
        }
    }

    async fn handle_deserialized_message(
        context: Arc<Self::Context>,
        deserialized_message: Self::Message,
        mut session: Session,
        _shutdown_tx: broadcast::Sender<()>,
    ) -> Result<ContinuationDecision> {
        match deserialized_message {
            InferenceJsonRpcMessage::Error(JsonRpcError {
                code,
                description,
            }) => {
                error!("Received error from client: code: {code}, description: {description:?}");

                return Ok(ContinuationDecision::Continue);
            }
            InferenceJsonRpcMessage::Request(RequestEnvelope {
                id,
                request: InferenceJsonRpcRequest::GenerateTokens(params),
            }) => {
                rt::spawn(async move {
                    let (generate_tokens_tx, mut generate_tokens_rx) = mpsc::channel(100);

                    let _guard = GenerateTokensSenderGuard {
                        request_id: id.clone(),
                        sender_collection: context.generate_tokens_sender_collection.clone(),
                    };

                    context
                        .generate_tokens_sender_collection
                        .register_sender(id.clone(), generate_tokens_tx);

                    if let Some(agent_controller) =
                        context.agent_controller_pool.find_best_agent_controller()
                    {
                        agent_controller
                            .send_rpc_message(AgentJsonRpcMessage::Request(RequestEnvelope {
                                id,
                                request: AgentJsonRpcRequest::GenerateTokens(params),
                            }))
                            .await
                            .unwrap_or_else(|err| {
                                error!("Failed to send GenerateTokens request: {err}");
                            });
                    } else {
                        error!("No agent controller found to handle GenerateTokens request");
                    }

                    while let Some(response) = generate_tokens_rx.recv().await {
                        match response {
                            ChunkResponse::Data(chunk) => {
                                println!("Received chunk: {chunk:?}");
                            }
                            ChunkResponse::Error(error) => {
                                session
                                    .text(
                                        match serde_json::to_string(
                                            &InferenceJsonRpcMessage::Error(JsonRpcError {
                                                code: 500,
                                                description: Some(error),
                                            }),
                                        ) {
                                            Ok(json) => json,
                                            Err(err) => {
                                                error!("Failed to serialize error response: {err}");

                                                return;
                                            }
                                        },
                                    )
                                    .await
                                    .unwrap_or_else(|err| {
                                        error!("Failed to send error response: {err}");
                                    });
                            }
                        }
                    }
                });

                return Ok(ContinuationDecision::Continue);
            }
        }
    }
}

#[get("/api/v1/inference_socket")]
async fn respond(
    agent_controller_pool: Data<AgentControllerPool>,
    generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    payload: Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let inference_socket_controller = InferenceSocketController {
        agent_controller_pool,
        generate_tokens_sender_collection,
    };

    inference_socket_controller.respond(payload, req)
}
