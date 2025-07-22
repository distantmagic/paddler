pub mod client;
mod inference_socket_controller_context;
pub mod jsonrpc;

use std::sync::Arc;

use actix_web::get;
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
use tokio::time::sleep;

use self::client::Message as OutgoingMessage;
use self::inference_socket_controller_context::InferenceSocketControllerContext;
use self::jsonrpc::Message as InferenceJsonRpcMessage;
use self::jsonrpc::Request as InferenceJsonRpcRequest;
use crate::balancer::buffered_request_agent_wait_result::BufferedRequestAgentWaitResult;
use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::balancer::inference_service::configuration::Configuration as InferenceServiceConfiguration;
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
    inference_service_configuration: Data<InferenceServiceConfiguration>,
}

#[async_trait]
impl ControlsWebSocketEndpoint for InferenceSocketController {
    type Context = InferenceSocketControllerContext;
    type IncomingMessage = InferenceJsonRpcMessage;
    type OutgoingMessage = OutgoingMessage;

    fn create_context(&self) -> Self::Context {
        InferenceSocketControllerContext {
            buffered_request_manager: self.buffered_request_manager.clone(),
            inference_service_configuration: self.inference_service_configuration.clone(),
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
                let mut respond_with_error = async |error: JsonRpcError| {
                    websocket_session_controller
                        .send_response(OutgoingMessage::Error(ErrorEnvelope {
                            request_id: id.clone(),
                            error,
                        }))
                        .await
                        .unwrap_or_else(|err| {
                            error!(
                                "Failed to send response for GenerateTokens request {id:?}: {err}"
                            );
                        });
                };

                tokio::select! {
                    _ = connection_close_rx.recv() => {
                        debug!("Connection close signal received, stopping GenerateTokens loop.");
                    },
                    buffered_request_agent_wait_result = buffered_request_manager.wait_for_available_agent() => {
                        match buffered_request_agent_wait_result {
                            Ok(BufferedRequestAgentWaitResult::Found(agent_controller)) => {
                                debug!("Found available agent controller for GenerateTokens request: {id:?}");

                                let mut agent_controller_connection_close_resubscribed = agent_controller.connection_close_rx.resubscribe();
                                let mut generated_tokens_controller = match agent_controller.generate_tokens(id.clone(), params).await {
                                    Ok(generated_tokens_controller) => generated_tokens_controller,
                                    Err(err) => {
                                        error!("Unable to start generate tokens controller for request {id:?}: {err}");

                                        respond_with_error(JsonRpcError {
                                            code: 500,
                                            description: "Internal server error".to_string(),
                                        }).await;

                                        return Ok(ContinuationDecision::Continue);
                                    }
                                };

                                loop {
                                    tokio::select! {
                                        _ = agent_controller_connection_close_resubscribed.recv() => {
                                            error!("Agent controller connection closed");
                                            respond_with_error(JsonRpcError {
                                                code: 502,
                                                description: "Agent controller connection closed".to_string(),
                                            }).await;
                                            break;
                                        }
                                        _ = connection_close_rx.recv() => {
                                            debug!("Connection close signal received");

                                            agent_controller.stop_generating_tokens(id.clone()).await.unwrap_or_else(|err| {
                                                error!("Failed to stop generating tokens for request {id:?}: {err}");
                                            });

                                            break;
                                        }
                                        _ = sleep(context.inference_service_configuration.inference_token_timeout) => {
                                            warn!("Timed out waiting for generated token");

                                            respond_with_error(JsonRpcError {
                                                code: 504,
                                                description: "Token generation timed out".to_string(),
                                            }).await;

                                            agent_controller.stop_generating_tokens(id.clone()).await.unwrap_or_else(|err| {
                                                error!("Failed to stop generating tokens for request {id:?}: {err}");
                                            });

                                            break;
                                        }
                                        generated_token = generated_tokens_controller.generated_tokens_rx.recv() => {
                                            match generated_token {
                                                Some(generated_token) => {
                                                    debug!("Received generated token for request {id:?}: {generated_token:?}");
                                                }
                                                None => break,
                                            }
                                        }
                                    }
                                }
                            }
                            Ok(BufferedRequestAgentWaitResult::BufferOverflow) => {
                                warn!("Too many buffered requests, dropping request: {id:?}");
                                respond_with_error(JsonRpcError {
                                    code: 503,
                                    description: "Buffered requests overflow".to_string(),
                                }).await;
                            }
                            Ok(BufferedRequestAgentWaitResult::Timeout(err)) => {
                                warn!("Buffered request {id:?} timed out: {err:?}");
                                respond_with_error(JsonRpcError {
                                    code: 408,
                                    description: "Waiting for available agent timed out".to_string(),
                                }).await;
                            }
                            Err(err) => {
                                error!("Error while waiting for available agent controller for GenerateTokens request: {err}");
                                respond_with_error(JsonRpcError {
                                    code: 500,
                                    description: "Internal server error".to_string(),
                                }).await;
                            }
                        }
                    }
                }

                debug!("GenerateTokens request processing completed for request: {id:?}");

                return Ok(ContinuationDecision::Continue);
            }
        }
    }
}

#[get("/api/v1/inference_socket")]
async fn respond(
    buffered_request_manager: Data<BufferedRequestManager>,
    inference_service_configuration: Data<InferenceServiceConfiguration>,
    payload: Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let inference_socket_controller = InferenceSocketController {
        buffered_request_manager,
        inference_service_configuration,
    };

    inference_socket_controller.respond(payload, req)
}
