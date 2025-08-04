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

use crate::balancer::inference_service::app_data::AppData;
use self::client::Message as OutgoingMessage;
use self::client::Response as OutgoingResponse;
use self::inference_socket_controller_context::InferenceSocketControllerContext;
use self::jsonrpc::Message as InferenceJsonRpcMessage;
use self::jsonrpc::Request as InferenceJsonRpcRequest;
use crate::balancer::agent_controller::AgentController;
use crate::balancer::buffered_request_agent_wait_result::BufferedRequestAgentWaitResult;
use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::balancer::inference_service::configuration::Configuration as InferenceServiceConfiguration;
use crate::balancer::receive_tokens_controller::ReceiveTokensController;
use crate::controls_websocket_endpoint::ContinuationDecision;
use crate::controls_websocket_endpoint::ControlsWebSocketEndpoint;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::generated_token_result::GeneratedTokenResult;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::ErrorEnvelope;
use crate::jsonrpc::RequestEnvelope;
use crate::jsonrpc::ResponseEnvelope;
use crate::websocket_session_controller::WebSocketSessionController;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

struct InferenceSocketController {
    buffered_request_manager: Arc<BufferedRequestManager>,
    inference_service_configuration: InferenceServiceConfiguration,
}

impl InferenceSocketController {
    async fn generate_tokens(
        agent_controller: Arc<AgentController>,
        mut connection_close_rx: broadcast::Receiver<()>,
        context: Arc<InferenceSocketControllerContext>,
        id: String,
        mut receive_tokens_controller: ReceiveTokensController,
        mut websocket_session_controller: WebSocketSessionController<OutgoingMessage>,
    ) -> Result<ContinuationDecision> {
        debug!("Found available agent controller for GenerateTokens request: {id:?}");

        let mut agent_controller_connection_close_resubscribed =
            agent_controller.connection_close_rx.resubscribe();

        loop {
            tokio::select! {
                _ = agent_controller_connection_close_resubscribed.recv() => {
                    error!("Agent controller connection closed");
                    Self::respond_with_error(
                        JsonRpcError {
                            code: 502,
                            description: "Agent controller connection closed".to_string(),
                        },
                        id.clone(),
                        &mut websocket_session_controller,
                    ).await;

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

                    Self::respond_with_error(
                        JsonRpcError {
                            code: 504,
                            description: "Token generation timed out".to_string(),
                        },
                        id.clone(),
                        &mut websocket_session_controller,
                    ).await;

                    agent_controller.stop_generating_tokens(id.clone()).await.unwrap_or_else(|err| {
                        error!("Failed to stop generating tokens for request {id:?}: {err}");
                    });

                    break;
                }
                generated_token_envelope = receive_tokens_controller.generated_tokens_rx.recv() => {
                    match generated_token_envelope {
                        Some(generated_token_envelope) => {
                            let is_done = matches!(generated_token_envelope.generated_token_result, GeneratedTokenResult::Done);

                            Self::respond_with_token(
                                generated_token_envelope,
                                id.clone(),
                                &mut websocket_session_controller,
                            ).await;

                            if is_done {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }

        Ok(ContinuationDecision::Continue)
    }

    async fn wait_for_agent_controller(
        mut connection_close_rx: broadcast::Receiver<()>,
        context: Arc<InferenceSocketControllerContext>,
        id: String,
        websocket_session_controller: &mut WebSocketSessionController<OutgoingMessage>,
    ) -> Result<Option<Arc<AgentController>>> {
        let buffered_request_manager = context.buffered_request_manager.clone();

        tokio::select! {
            _ = connection_close_rx.recv() => {
                debug!("Connection close signal received, stopping GenerateTokens loop.");

                Ok(None)
            },
            buffered_request_agent_wait_result = buffered_request_manager.wait_for_available_agent() => {
                match buffered_request_agent_wait_result {
                    Ok(BufferedRequestAgentWaitResult::Found(agent_controller)) => Ok(Some(agent_controller)),
                    Ok(BufferedRequestAgentWaitResult::BufferOverflow) => {
                        warn!("Too many buffered requests, dropping request: {id:?}");
                        Self::respond_with_error(
                            JsonRpcError {
                                code: 509,
                                description: "Buffered requests overflow".to_string(),
                            },
                            id.clone(),
                            websocket_session_controller,
                        ).await;

                        Ok(None)
                    }
                    Ok(BufferedRequestAgentWaitResult::Timeout(err)) => {
                        warn!("Buffered request {id:?} timed out: {err:?}");
                        Self::respond_with_error(
                            JsonRpcError {
                                code: 503,
                                description: "Waiting for available slot timed out".to_string(),
                            },
                            id.clone(),
                            websocket_session_controller,
                        ).await;

                        Ok(None)
                    }
                    Err(err) => {
                        error!("Error while waiting for available agent controller for GenerateTokens request: {err}");
                        Self::respond_with_error(
                            JsonRpcError {
                                code: 500,
                                description: "Internal server error".to_string(),
                            },
                            id.clone(),
                            websocket_session_controller,
                        ).await;

                        Ok(None)
                    }
                }
            }
        }
    }

    async fn respond_with_error(
        error: JsonRpcError,
        request_id: String,
        websocket_session_controller: &mut WebSocketSessionController<OutgoingMessage>,
    ) {
        websocket_session_controller
            .send_response(OutgoingMessage::Error(ErrorEnvelope {
                request_id: request_id.clone(),
                error,
            }))
            .await
            .unwrap_or_else(|err| {
                error!("Failed to send response for GenerateTokens request {request_id:?}: {err}");
            });
    }

    async fn respond_with_token(
        generated_token_envelope: GeneratedTokenEnvelope,
        request_id: String,
        websocket_session_controller: &mut WebSocketSessionController<OutgoingMessage>,
    ) {
        websocket_session_controller
            .send_response(OutgoingMessage::Response(ResponseEnvelope {
                request_id: request_id.clone(),
                response: OutgoingResponse::GeneratedToken(generated_token_envelope),
            }))
            .await
            .unwrap_or_else(|err| {
                error!("Failed to send generated token for request {request_id:?}: {err}");
            });
    }
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
                request: InferenceJsonRpcRequest::ContinueFromConversationHistory(params),
            }) => {
                debug!(
                    "Received continue from conversation history request from client: {id:?}, params: {params:?}"
                );

                match Self::wait_for_agent_controller(
                    connection_close_tx.subscribe(),
                    context.clone(),
                    id.clone(),
                    &mut websocket_session_controller,
                )
                .await?
                {
                    Some(agent_controller) => {
                        let receive_tokens_controller = match agent_controller
                            .continue_from_conversation_history(id.clone(), params)
                            .await
                        {
                            Ok(receive_tokens_controller) => receive_tokens_controller,
                            Err(err) => {
                                error!("Failed to continue conversation for request {id:?}: {err}");

                                Self::respond_with_error(
                                    JsonRpcError {
                                        code: 500,
                                        description: "Failed to continue conversation".to_string(),
                                    },
                                    id.clone(),
                                    &mut websocket_session_controller,
                                )
                                .await;

                                return Ok(ContinuationDecision::Continue);
                            }
                        };

                        Self::generate_tokens(
                            agent_controller,
                            connection_close_tx.subscribe(),
                            context,
                            id,
                            receive_tokens_controller,
                            websocket_session_controller,
                        )
                        .await
                    }
                    None => Ok(ContinuationDecision::Continue),
                }
            }
            InferenceJsonRpcMessage::Request(RequestEnvelope {
                id,
                request: InferenceJsonRpcRequest::ContinueFromRawPrompt(params),
            }) => {
                debug!("Received GenerateTokens request from client: {id:?}, params: {params:?}");

                match Self::wait_for_agent_controller(
                    connection_close_tx.subscribe(),
                    context.clone(),
                    id.clone(),
                    &mut websocket_session_controller,
                )
                .await?
                {
                    Some(agent_controller) => {
                        let receive_tokens_controller =
                            match agent_controller.continue_from_raw_prompt(id.clone(), params).await {
                                Ok(receive_tokens_controller) => receive_tokens_controller,
                                Err(err) => {
                                    error!("Failed to generate tokens: {err}");

                                    Self::respond_with_error(
                                        JsonRpcError {
                                            code: 500,
                                            description: "Failed to generate tokens".to_string(),
                                        },
                                        id.clone(),
                                        &mut websocket_session_controller,
                                    )
                                    .await;

                                    return Ok(ContinuationDecision::Continue);
                                }
                            };

                        Self::generate_tokens(
                            agent_controller,
                            connection_close_tx.subscribe(),
                            context,
                            id,
                            receive_tokens_controller,
                            websocket_session_controller,
                        )
                        .await
                    }
                    None => Ok(ContinuationDecision::Continue),
                }
            }
        }
    }
}

#[get("/api/v1/inference_socket")]
async fn respond(
    app_data: Data<AppData>,
    payload: Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let inference_socket_controller = InferenceSocketController {
        buffered_request_manager: app_data.buffered_request_manager.clone(),
        inference_service_configuration: app_data.inference_service_configuration.clone(),
    };

    inference_socket_controller.respond(payload, req)
}
