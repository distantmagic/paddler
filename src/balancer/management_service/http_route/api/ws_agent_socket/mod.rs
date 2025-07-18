pub mod jsonrpc;

use std::sync::Arc;

use actix_web::get;
use actix_web::rt;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::web::Payload;
use actix_web::web::ServiceConfig;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_ws::Session;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use log::info;
use serde::Deserialize;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use self::jsonrpc::notification_params::RegisterAgentParams;
use self::jsonrpc::notification_params::UpdateAgentStatusParams;
use self::jsonrpc::Message as ManagementJsonRpcMessage;
use self::jsonrpc::Notification as ManagementJsonRpcNotification;
use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::notification_params::VersionParams;
use crate::agent::jsonrpc::Message as AgentJsonRpcMessage;
use crate::agent::jsonrpc::Notification as AgentJsonRpcNotification;
use crate::agent::jsonrpc::Response as AgentJsonRpcResponse;
use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller::AgentController;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::state_database::StateDatabase;
use crate::controls_websocket_endpoint::ContinuationDecision;
use crate::controls_websocket_endpoint::ControlsWebSocketEndpoint;
use crate::jsonrpc::ResponseEnvelope;
use crate::response::ChunkResponse;
use crate::response_params::GeneratedToken;
use crate::sends_rpc_message::SendsRpcMessage as _;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

struct AgentSocketControllerContext {
    agent_controller_pool: Data<AgentControllerPool>,
    agent_id: String,
    generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    state_database: Data<dyn StateDatabase>,
}

impl Drop for AgentSocketControllerContext {
    fn drop(&mut self) {
        if let Err(err) = self
            .agent_controller_pool
            .remove_agent_controller(&self.agent_id)
        {
            error!("Failed to remove agent: {err}");
        }

        info!("Removed agent: {}", self.agent_id);
    }
}

struct AgentSocketController {
    agent_controller_pool: Data<AgentControllerPool>,
    agent_id: String,
    generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    state_database: Data<dyn StateDatabase>,
}

#[async_trait]
impl ControlsWebSocketEndpoint for AgentSocketController {
    type Context = AgentSocketControllerContext;
    type Message = ManagementJsonRpcMessage;

    fn create_context(&self) -> Self::Context {
        AgentSocketControllerContext {
            agent_controller_pool: self.agent_controller_pool.clone(),
            agent_id: self.agent_id.clone(),
            generate_tokens_sender_collection: self.generate_tokens_sender_collection.clone(),
            state_database: self.state_database.clone(),
        }
    }

    async fn handle_deserialized_message(
        context: Arc<Self::Context>,
        deserialized_message: Self::Message,
        mut session: Session,
        shutdown_tx: broadcast::Sender<()>,
    ) -> Result<ContinuationDecision> {
        match deserialized_message {
            ManagementJsonRpcMessage::Error(err) => {
                error!("Received error message: {err:?}");

                Ok(ContinuationDecision::Continue)
            }
            ManagementJsonRpcMessage::Notification(
                ManagementJsonRpcNotification::DeregisterAgent,
            ) => {
                shutdown_tx.send(())?;

                return Ok(ContinuationDecision::Stop);
            }
            ManagementJsonRpcMessage::Notification(
                ManagementJsonRpcNotification::RegisterAgent(RegisterAgentParams {
                    name,
                    slots_total,
                }),
            ) => {
                let (agent_tx, mut agent_rx) = mpsc::channel(1000);
                let agent_controller = AgentController {
                    agent_tx,
                    id: context.agent_id.clone(),
                    name,
                    slots_processing: AtomicValue::new(0),
                    slots_total,
                };

                if let Some(desired_state) = context.state_database.read_desired_state().await? {
                    agent_controller
                        .send_rpc_message(AgentJsonRpcMessage::Notification(
                            AgentJsonRpcNotification::SetState(SetStateParams {
                                desired_state,
                            }),
                        ))
                        .await
                        .context("Unable to set desired state")?;
                }

                context
                    .agent_controller_pool
                    .register_agent_controller(context.agent_id.clone(), Arc::new(agent_controller))
                    .context("Unable to register agent controller")?;

                info!("Registered agent: {}", context.agent_id);

                let mut shutdown_tx_resubscribed = shutdown_tx.subscribe();

                rt::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = shutdown_tx_resubscribed.recv() => {
                                break;
                            }
                            result = agent_rx.recv() => {
                                match result {
                                    Some(text) => {
                                        if let Err(err) = session.text(text).await {
                                            error!("Error sending message to agent: {err:?}");
                                            break;
                                        }
                                    }
                                    None => {
                                        info!("Session channel closed for agent: {}", context.agent_id);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                });

                Ok(ContinuationDecision::Continue)
            }
            ManagementJsonRpcMessage::Notification(
                ManagementJsonRpcNotification::UpdateAgentStatus(UpdateAgentStatusParams {
                    slots_processing,
                }),
            ) => {
                if let Some(agent_controller) = context
                    .agent_controller_pool
                    .get_agent_controller(&context.agent_id)
                {
                    agent_controller.slots_processing.set(slots_processing);
                    context
                        .agent_controller_pool
                        .update_notifier
                        .notify_waiters();
                } else {
                    error!("Agent controller not found for agent: {}", context.agent_id);
                }

                Ok(ContinuationDecision::Continue)
            }
            ManagementJsonRpcMessage::Response(ResponseEnvelope::Error {
                request_id,
                error,
            }) => {
                context
                    .generate_tokens_sender_collection
                    .forward_response(request_id, ChunkResponse::Error(error))
                    .await?;

                Ok(ContinuationDecision::Continue)
            }
            ManagementJsonRpcMessage::Response(ResponseEnvelope::StreamChunk {
                request_id,
                chunk:
                    AgentJsonRpcResponse::GeneratedToken(GeneratedToken {
                        token,
                    }),
            }) => {
                context
                    .generate_tokens_sender_collection
                    .forward_response(
                        request_id,
                        ChunkResponse::Data(GeneratedToken {
                            token,
                        }),
                    )
                    .await?;

                Ok(ContinuationDecision::Continue)
            }
            ManagementJsonRpcMessage::Response(ResponseEnvelope::StreamDone {
                request_id,
            }) => {
                println!("Stream done: {request_id}");
                Ok(ContinuationDecision::Continue)
            }
        }
    }

    async fn handle_serialization_error(
        _context: Arc<Self::Context>,
        error: serde_json::Error,
        _session: Session,
        _shutdown_tx: broadcast::Sender<()>,
    ) -> Result<ContinuationDecision> {
        error!("Error in AgentSocketController: {error}");

        Ok(ContinuationDecision::Continue)
    }

    async fn on_connection_start(
        _context: Arc<Self::Context>,
        session: &mut Session,
    ) -> Result<ContinuationDecision> {
        if let Err(err) = session
            .text(serde_json::to_string(&AgentJsonRpcMessage::Notification(
                AgentJsonRpcNotification::Version(VersionParams {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                }),
            ))?)
            .await
        {
            error!("Error sending version: {err:?}");

            return Ok(ContinuationDecision::Stop);
        }

        Ok(ContinuationDecision::Continue)
    }
}

#[derive(Deserialize)]
struct PathParams {
    agent_id: String,
}

#[get("/api/v1/agent_socket/{agent_id}")]
async fn respond(
    agent_controller_pool: Data<AgentControllerPool>,
    generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    state_database: Data<dyn StateDatabase>,
    path_params: Path<PathParams>,
    payload: Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let agent_socket_controller = AgentSocketController {
        agent_controller_pool,
        agent_id: path_params.agent_id.clone(),
        generate_tokens_sender_collection,
        state_database,
    };

    agent_socket_controller.respond(payload, req)
}
