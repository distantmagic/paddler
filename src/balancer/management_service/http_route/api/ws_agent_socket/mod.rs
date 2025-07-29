mod agent_socket_controller_context;
pub mod jsonrpc;

use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicUsize;
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
use log::warn;
use serde::Deserialize;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use self::agent_socket_controller_context::AgentSocketControllerContext;
use self::jsonrpc::notification_params::RegisterAgentParams;
use self::jsonrpc::notification_params::UpdateAgentStatusParams;
use self::jsonrpc::Message as ManagementJsonRpcMessage;
use self::jsonrpc::Notification as ManagementJsonRpcNotification;
use crate::agent::jsonrpc::notification_params::VersionParams;
use crate::agent::jsonrpc::Message as AgentJsonRpcMessage;
use crate::agent::jsonrpc::Notification as AgentJsonRpcNotification;
use crate::agent::jsonrpc::Response as AgentJsonRpcResponse;
use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller::AgentController;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::agent_controller_update_result::AgentControllerUpdateResult;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::model_metadata_sender_collection::ModelMetadataSenderCollection;
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

struct AgentSocketController {
    agent_controller_pool: Data<AgentControllerPool>,
    agent_id: String,
    generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    model_metadata_sender_collection: Data<ModelMetadataSenderCollection>,
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
            generate_tokens_sender_collection: self.generate_tokens_sender_collection.clone(),
            model_metadata_sender_collection: self.model_metadata_sender_collection.clone(),
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
                            download_current,
                            download_filename,
                            download_total,
                            issues,
                            model_path,
                            slots_processing,
                            slots_total,
                            version,
                        },
                }),
            ) => {
                let (agent_message_tx, mut agent_message_rx) =
                    mpsc::unbounded_channel::<AgentJsonRpcMessage>();
                let agent_controller = Arc::new(AgentController {
                    agent_message_tx,
                    connection_close_rx: connection_close_tx.subscribe(),
                    desired_slots_total: AtomicValue::<AtomicI32>::new(desired_slots_total),
                    download_current: AtomicValue::<AtomicUsize>::new(download_current),
                    download_filename: RwLock::new(download_filename),
                    download_total: AtomicValue::<AtomicUsize>::new(download_total),
                    generate_tokens_sender_collection: context
                        .generate_tokens_sender_collection
                        .clone(),
                    model_metadata_sender_collection: context
                        .model_metadata_sender_collection
                        .clone(),
                    id: context.agent_id.clone(),
                    issues: RwLock::new(issues),
                    model_path: RwLock::new(model_path),
                    name,
                    newest_update_version: AtomicValue::<AtomicI32>::new(version),
                    slots_processing: AtomicValue::<AtomicI32>::new(slots_processing),
                    slots_total: AtomicValue::<AtomicI32>::new(slots_total),
                });

                context
                    .agent_controller_pool
                    .register_agent_controller(context.agent_id.clone(), agent_controller.clone())
                    .context("Unable to register agent controller")?;

                let desired_state = context
                    .state_database
                    .read_desired_state()
                    .await?
                    .unwrap_or_default();

                agent_controller
                    .set_desired_state(desired_state)
                    .await
                    .context("Unable to set desired state")?;

                info!("Registered agent: {}", context.agent_id);

                let mut shutdown_tx_resubscribed = connection_close_tx.subscribe();

                rt::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = shutdown_tx_resubscribed.recv() => {
                                break;
                            }
                            result = agent_message_rx.recv() => {
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
                    slot_aggregated_status_snapshot,
                }),
            ) => {
                if let Some(agent_controller) = context
                    .agent_controller_pool
                    .get_agent_controller(&context.agent_id)
                {
                    match agent_controller.update_from_slot_aggregated_status_snapshot(
                        slot_aggregated_status_snapshot,
                    ) {
                        AgentControllerUpdateResult::NoMeaningfulChanges => {}
                        AgentControllerUpdateResult::Updated => {
                            context
                                .agent_controller_pool
                                .update_notifier
                                .notify_waiters();
                        }
                    }
                } else {
                    error!("Agent controller not found for agent: {}", context.agent_id);
                }

                Ok(ContinuationDecision::Continue)
            }
            ManagementJsonRpcMessage::Response(ResponseEnvelope {
                request_id,
                response: AgentJsonRpcResponse::GeneratedToken(generated_token_envelope),
            }) => {
                if let Err(err) = context
                    .generate_tokens_sender_collection
                    .forward_generated_token(request_id, generated_token_envelope)
                    .await
                {
                    // Token might come in after the sender was deregistered
                    warn!("Error forwarding generated token: {err}");
                }

                Ok(ContinuationDecision::Continue)
            }
            ManagementJsonRpcMessage::Response(ResponseEnvelope {
                request_id,
                response: AgentJsonRpcResponse::ModelMetadata(model_metadata),
            }) => {
                if let Err(err) = context
                    .model_metadata_sender_collection
                    .forward_model_metadata(request_id, model_metadata)
                    .await
                {
                    // Metadata might come in after awaiting connection is closed
                    warn!("Error forwarding model metadata: {err}");
                }

                Ok(ContinuationDecision::Continue)
            }
        }
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
    model_metadata_sender_collection: Data<ModelMetadataSenderCollection>,
    state_database: Data<dyn StateDatabase>,
    path_params: Path<PathParams>,
    payload: Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let agent_socket_controller = AgentSocketController {
        agent_controller_pool,
        agent_id: path_params.agent_id.clone(),
        generate_tokens_sender_collection,
        model_metadata_sender_collection,
        state_database,
    };

    agent_socket_controller.respond(payload, req)
}
