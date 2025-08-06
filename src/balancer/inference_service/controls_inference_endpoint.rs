use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use log::error;
use log::warn;
use tokio::sync::broadcast;
use tokio::time::sleep;

use crate::balancer::agent_controller::AgentController;
use crate::balancer::buffered_request_agent_wait_result::BufferedRequestAgentWaitResult;
use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::balancer::inference_service::configuration::Configuration as InferenceServiceConfiguration;
use crate::balancer::inference_service::http_route::api::ws_inference_socket::client::Message as OutgoingMessage;
use crate::balancer::inference_service::http_route::api::ws_inference_socket::client::Response as OutgoingResponse;
use crate::balancer::receive_tokens_controller::ReceiveTokensController;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::generated_token_result::GeneratedTokenResult;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::ErrorEnvelope;
use crate::jsonrpc::ResponseEnvelope;
use crate::request_params::ContinueFromConversationHistoryParams;
use crate::request_params::ContinueFromRawPromptParams;
use crate::session_controller::SessionController;

#[async_trait]
pub trait ControlsInferenceEndpoint {
    type SessionController: SessionController<OutgoingMessage>;

    async fn continue_from_conversation_history(
        buffered_request_manager: Arc<BufferedRequestManager>,
        connection_close_tx: broadcast::Sender<()>,
        inference_service_configuration: InferenceServiceConfiguration,
        params: ContinueFromConversationHistoryParams,
        request_id: String,
        mut session_controller: Self::SessionController,
    ) -> Result<()> {
        debug!("Received continue from conversation history request from client: {request_id:?}, params: {params:?}");

        match Self::wait_for_agent_controller(
            buffered_request_manager.clone(),
            connection_close_tx.subscribe(),
            request_id.clone(),
            &mut session_controller,
        )
        .await?
        {
            Some(agent_controller) => {
                let receive_tokens_controller = match agent_controller
                    .continue_from_conversation_history(request_id.clone(), params)
                    .await
                {
                    Ok(receive_tokens_controller) => receive_tokens_controller,
                    Err(err) => {
                        error!("Failed to continue conversation for request {request_id:?}: {err}");

                        Self::respond_with_error(
                            JsonRpcError {
                                code: 500,
                                description: "Failed to continue conversation".to_string(),
                            },
                            request_id.clone(),
                            &mut session_controller,
                        )
                        .await;

                        return Ok(());
                    }
                };

                Self::generate_tokens(
                    agent_controller,
                    connection_close_tx.subscribe(),
                    inference_service_configuration,
                    receive_tokens_controller,
                    request_id,
                    session_controller,
                )
                .await?;

                Ok(())
            }
            None => Ok(()),
        }
    }

    async fn continue_from_raw_prompt(
        buffered_request_manager: Arc<BufferedRequestManager>,
        connection_close_tx: broadcast::Sender<()>,
        inference_service_configuration: InferenceServiceConfiguration,
        params: ContinueFromRawPromptParams,
        request_id: String,
        mut session_controller: Self::SessionController,
    ) -> Result<()> {
        debug!("Received GenerateTokens request from client: {request_id:?}, params: {params:?}");

        match Self::wait_for_agent_controller(
            buffered_request_manager.clone(),
            connection_close_tx.subscribe(),
            request_id.clone(),
            &mut session_controller,
        )
        .await?
        {
            Some(agent_controller) => {
                let receive_tokens_controller = match agent_controller
                    .continue_from_raw_prompt(request_id.clone(), params)
                    .await
                {
                    Ok(receive_tokens_controller) => receive_tokens_controller,
                    Err(err) => {
                        error!("Failed to generate tokens: {err}");

                        Self::respond_with_error(
                            JsonRpcError {
                                code: 500,
                                description: "Failed to generate tokens".to_string(),
                            },
                            request_id.clone(),
                            &mut session_controller,
                        )
                        .await;

                        return Ok(());
                    }
                };

                Self::generate_tokens(
                    agent_controller,
                    connection_close_tx.subscribe(),
                    inference_service_configuration,
                    receive_tokens_controller,
                    request_id,
                    session_controller,
                )
                .await?;

                Ok(())
            }
            None => Ok(()),
        }
    }

    async fn generate_tokens(
        agent_controller: Arc<AgentController>,
        mut connection_close_rx: broadcast::Receiver<()>,
        inference_service_configuration: InferenceServiceConfiguration,
        mut receive_tokens_controller: ReceiveTokensController,
        request_id: String,
        mut session_controller: Self::SessionController,
    ) -> Result<()> {
        debug!("Found available agent controller for GenerateTokens request: {request_id:?}");

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
                        request_id,
                        &mut session_controller,
                    ).await;

                    break;
                }
                _ = connection_close_rx.recv() => {
                    debug!("Connection close signal received");

                    agent_controller.stop_generating_tokens(request_id.clone()).await.unwrap_or_else(|err| {
                        error!("Failed to stop generating tokens for request {request_id:?}: {err}");
                    });

                    break;
                }
                _ = sleep(inference_service_configuration.inference_token_timeout) => {
                    warn!("Timed out waiting for generated token");

                    Self::respond_with_error(
                        JsonRpcError {
                            code: 504,
                            description: "Token generation timed out".to_string(),
                        },
                        request_id.clone(),
                        &mut session_controller,
                    ).await;

                    agent_controller.stop_generating_tokens(request_id.clone()).await.unwrap_or_else(|err| {
                        error!("Failed to stop generating tokens for request {request_id:?}: {err}");
                    });

                    break;
                }
                generated_token_envelope = receive_tokens_controller.generated_tokens_rx.recv() => {
                    match generated_token_envelope {
                        Some(generated_token_envelope) => {
                            let is_done = matches!(generated_token_envelope.generated_token_result, GeneratedTokenResult::Done);

                            Self::respond_with_token(
                                agent_controller.clone(),
                                generated_token_envelope,
                                request_id.clone(),
                                &mut session_controller,
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

        Ok(())
    }

    async fn respond_with_error(
        error: JsonRpcError,
        request_id: String,
        session_controller: &mut Self::SessionController,
    ) {
        session_controller
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
        agent_controller: Arc<AgentController>,
        generated_token_envelope: GeneratedTokenEnvelope,
        request_id: String,
        session_controller: &mut Self::SessionController,
    ) {
        if let Err(err) = session_controller
            .send_response(OutgoingMessage::Response(ResponseEnvelope {
                request_id: request_id.clone(),
                response: OutgoingResponse::GeneratedToken(generated_token_envelope),
            }))
            .await
        {
            error!("Failed to send generated token for request {request_id:?}: {err}");

            agent_controller
                .stop_generating_tokens(request_id.clone())
                .await
                .unwrap_or_else(|err| {
                    error!("Failed to stop generating tokens for request {request_id:?}: {err}");
                });
        }
    }

    async fn wait_for_agent_controller(
        buffered_request_manager: Arc<BufferedRequestManager>,
        mut connection_close_rx: broadcast::Receiver<()>,
        request_id: String,
        session_controller: &mut Self::SessionController,
    ) -> Result<Option<Arc<AgentController>>> {
        let buffered_request_manager = buffered_request_manager.clone();

        tokio::select! {
            _ = connection_close_rx.recv() => {
                debug!("Connection close signal received, stopping GenerateTokens loop.");

                Ok(None)
            },
            buffered_request_agent_wait_result = buffered_request_manager.wait_for_available_agent() => {
                match buffered_request_agent_wait_result {
                    Ok(BufferedRequestAgentWaitResult::Found(agent_controller)) => Ok(Some(agent_controller)),
                    Ok(BufferedRequestAgentWaitResult::BufferOverflow) => {
                        warn!("Too many buffered requests, dropping request: {request_id:?}");
                        Self::respond_with_error(
                            JsonRpcError {
                                code: 503,
                                description: "Buffered requests overflow".to_string(),
                            },
                            request_id.clone(),
                            session_controller,
                        ).await;

                        Ok(None)
                    }
                    Ok(BufferedRequestAgentWaitResult::Timeout(err)) => {
                        warn!("Buffered request {request_id:?} timed out: {err:?}");
                        Self::respond_with_error(
                            JsonRpcError {
                                code: 504,
                                description: "Waiting for available slot timed out".to_string(),
                            },
                            request_id.clone(),
                            session_controller,
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
                            request_id.clone(),
                            session_controller,
                        ).await;

                        Ok(None)
                    }
                }
            }
        }
    }
}
