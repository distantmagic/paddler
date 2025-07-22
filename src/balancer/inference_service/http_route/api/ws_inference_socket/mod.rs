pub mod client;
mod inference_socket_controller_context;
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
use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use log::error;
use log::warn;
use tokio::sync::broadcast;

use self::client::Message as OutgoingMessage;
use self::inference_socket_controller_context::InferenceSocketControllerContext;
use self::jsonrpc::Message as InferenceJsonRpcMessage;
use self::jsonrpc::Request as InferenceJsonRpcRequest;
use crate::balancer::buffered_request_agent_wait_result::BufferedRequestAgentWaitResult;
use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::controls_websocket_endpoint::ContinuationDecision;
use crate::controls_websocket_endpoint::ControlsWebSocketEndpoint;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::ErrorEnvelope;
use crate::jsonrpc::RequestEnvelope;
use crate::websocket_session_controller::WebSocketSessionController;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

struct InferenceSocketController {
    buffered_request_manager: Data<BufferedRequestManager>,
}

#[async_trait]
impl ControlsWebSocketEndpoint for InferenceSocketController {
    type Context = InferenceSocketControllerContext;
    type IncomingMessage = InferenceJsonRpcMessage;
    type OutgoingMessage = OutgoingMessage;

    fn create_context(&self) -> Self::Context {
        InferenceSocketControllerContext {
            buffered_request_manager: self.buffered_request_manager.clone(),
        }
    }

    async fn handle_deserialized_message(
        connection_close_tx: broadcast::Sender<()>,
        context: Arc<Self::Context>,
        deserialized_message: Self::IncomingMessage,
        mut websocket_session_controller: WebSocketSessionController<Self::OutgoingMessage>,
    ) -> Result<ContinuationDecision> {
        match deserialized_message {
            InferenceJsonRpcMessage::Error(ErrorEnvelope {
                request_id,
                error: JsonRpcError { code, description },
            }) => {
                error!("Received error from client: code: {code}, description: {description:?}, request_id: {request_id:?}");

                return Ok(ContinuationDecision::Continue);
            }
            InferenceJsonRpcMessage::Request(RequestEnvelope {
                id,
                request: InferenceJsonRpcRequest::GenerateTokens(params),
            }) => {
                debug!("Received GenerateTokens request from client: {id:?}, params: {params:?}");

                let buffered_request_manager = context.buffered_request_manager.clone();
                let mut connection_close_rx = connection_close_tx.subscribe();

                rt::spawn(async move {
                    tokio::select! {
                        _ = connection_close_rx.recv() => {
                            debug!("Connection close signal received, stopping GenerateTokens loop.");
                        },
                        buffered_request_agent_wait_result = buffered_request_manager.wait_for_available_agent() => {
                            match buffered_request_agent_wait_result {
                                Ok(BufferedRequestAgentWaitResult::Found(agent_controller)) => {
                                    debug!("Found available agent controller for GenerateTokens request: {id:?}");
                                }
                                Ok(BufferedRequestAgentWaitResult::BufferOverflow) => {
                                    warn!("Too many buffered requests, dropping request: {id:?}");
                                    websocket_session_controller
                                        .send_response(OutgoingMessage::Error(ErrorEnvelope {
                                            request_id: id.clone(),
                                            error: JsonRpcError {
                                                code: 503,
                                                description: "Buffered requests overflow".to_string(),
                                            },
                                        }))
                                        .await
                                        .unwrap_or_else(|err| {
                                            error!("Failed to send response for GenerateTokens request {id:?}: {err}");
                                        });
                                }
                                Ok(BufferedRequestAgentWaitResult::Timeout(err)) => {
                                    warn!("Buffered request {id:?} timed out: {err:?}");
                                }
                                Err(err) => {
                                    error!("Error while waiting for available agent controller for GenerateTokens request: {err}");
                                }
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
    buffered_request_manager: Data<BufferedRequestManager>,
    payload: Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let inference_socket_controller = InferenceSocketController {
        buffered_request_manager,
    };

    inference_socket_controller.respond(payload, req)
}
