pub mod jsonrpc;
use std::sync::Arc;
use std::sync::RwLock;

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
use crate::agent::jsonrpc::notification_params::VersionParams;
use crate::agent::jsonrpc::Message as AgentJsonRpcMessage;
use crate::agent::jsonrpc::Notification as AgentJsonRpcNotification;
use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller::AgentController;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::state_database::StateDatabase;
use crate::controls_websocket_endpoint::ContinuationDecision;
use crate::controls_websocket_endpoint::ControlsWebSocketEndpoint;
use crate::jsonrpc::ResponseEnvelope;
use crate::sets_desired_state::SetsDesiredState as _;
use crate::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;
use crate::websocket_session_controller::WebSocketSessionController;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

struct AgentSocketControllerContext {
    agent_controller_pool: Data<AgentControllerPool>,
    agent_id: String,
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
    state_database: Data<dyn StateDatabase>,
}

#[async_trait]
impl ControlsWebSocketEndpoint for AgentSocketController {
    type Context = AgentSocketControllerContext;
    type IncomingMessage = ManagementJsonRpcMessage;
    type OutgoingMessage = AgentJsonRpcMessage;

    fn create_context(&self) -> Self::Context {
        AgentSocketControllerContext {
            agent_controller_pool: self.agent_controller_pool.clone(),
            agent_id: self.agent_id.clone(),
            state_database: self.state_database.clone(),
        }
    }

    async fn handle_deserialized_message(
        connection_close_tx: broadcast::Sender<()>,
        context: Arc<Self::Context>,
        deserialized_message: Self::IncomingMessage,
        mut websocket_session_controller: WebSocketSessionController<Self::OutgoingMessage>,
    ) -> Result<ContinuationDecision> {
        match deserialized_message {
            ManagementJsonRpcMessage::Error(err) => {
                error!("Received error message: {err:?}");

                Ok(ContinuationDecision::Continue)
            }
            ManagementJsonRpcMessage::Notification(
                ManagementJsonRpcNotification::DeregisterAgent,
            ) => {
                connection_close_tx.send(())?;

                return Ok(ContinuationDecision::Stop);
            }
            ManagementJsonRpcMessage::Notification(
                ManagementJsonRpcNotification::RegisterAgent(RegisterAgentParams {
                    name,
                    slot_aggregated_status_snapshot:
                        SlotAggregatedStatusSnapshot {
                            desired_slots_total,
                            model_path,
                            slots_processing,
                            slots_total,
                        },
                }),
            ) => {
                let (agent_tx, mut agent_rx) = mpsc::channel::<AgentJsonRpcMessage>(1000);
                let agent_controller = AgentController {
                    agent_tx,
                    desired_slots_total: AtomicValue::new(desired_slots_total),
                    id: context.agent_id.clone(),
                    model_path: RwLock::new(model_path),
                    name,
                    slots_processing: AtomicValue::new(slots_processing),
                    slots_total: AtomicValue::new(slots_total),
                };

                if let Some(desired_state) = context.state_database.read_desired_state().await? {
                    agent_controller
                        .set_desired_state(desired_state)
                        .await
                        .context("Unable to set desired state")?;
                }

                context
                    .agent_controller_pool
                    .register_agent_controller(context.agent_id.clone(), Arc::new(agent_controller))
                    .context("Unable to register agent controller")?;

                info!("Registered agent: {}", context.agent_id);

                let mut shutdown_tx_resubscribed = connection_close_tx.subscribe();

                rt::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = shutdown_tx_resubscribed.recv() => {
                                break;
                            }
                            result = agent_rx.recv() => {
                                match result {
                                    Some(message) => {
                                        websocket_session_controller
                                            .send_response(message)
                                            .await
                                            .unwrap_or_else(|err| {
                                                error!("Error sending response: {err}");
                                            });
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
                    slot_aggregated_status_snapshot:
                        SlotAggregatedStatusSnapshot {
                            desired_slots_total,
                            model_path,
                            slots_processing,
                            slots_total,
                        },
                }),
            ) => {
                if let Some(agent_controller) = context
                    .agent_controller_pool
                    .get_agent_controller(&context.agent_id)
                {
                    agent_controller
                        .desired_slots_total
                        .set(desired_slots_total);
                    agent_controller.set_model_path(model_path);
                    agent_controller.slots_processing.set(slots_processing);
                    agent_controller.slots_total.set(slots_total);
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
                request_id: _,
                error: _,
            }) => Ok(ContinuationDecision::Continue),
            ManagementJsonRpcMessage::Response(ResponseEnvelope::StreamDone {
                request_id,
            }) => {
                println!("Stream done: {request_id}");
                Ok(ContinuationDecision::Continue)
            }
        }
    }

    async fn handle_serialization_error(
        _connection_close_tx: broadcast::Sender<()>,
        _context: Arc<Self::Context>,
        error: serde_json::Error,
        _websocket_session_controller: WebSocketSessionController<Self::OutgoingMessage>,
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
    state_database: Data<dyn StateDatabase>,
    path_params: Path<PathParams>,
    payload: Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let agent_socket_controller = AgentSocketController {
        agent_controller_pool,
        agent_id: path_params.agent_id.clone(),
        state_database,
    };

    agent_socket_controller.respond(payload, req)
}
