pub mod jsonrpc;

use std::sync::Arc;

use actix_web::get;
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

use self::jsonrpc::notification_params::RegisterAgentParams;
use self::jsonrpc::notification_params::UpdateAgentStatusParams;
use self::jsonrpc::Notification as BalancerJsonRpcNotification;
use crate::agent::jsonrpc::notification_params::VersionParams;
use crate::agent::jsonrpc::Notification as AgentJsonRpcNotification;
use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller::AgentController;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::state_database::StateDatabase;
use crate::controls_websocket_endpoint::ContinuationDecision;
use crate::controls_websocket_endpoint::ControlsWebSocketEndpoint;

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
    type Message = BalancerJsonRpcNotification;

    fn create_context(&self) -> Self::Context {
        AgentSocketControllerContext {
            agent_controller_pool: self.agent_controller_pool.clone(),
            agent_id: self.agent_id.clone(),
            state_database: self.state_database.clone(),
        }
    }

    async fn handle_deserialized_message(
        context: Arc<Self::Context>,
        deserialized_message: Self::Message,
        session: Session,
        shutdown_tx: broadcast::Sender<()>,
    ) -> Result<ContinuationDecision> {
        match deserialized_message {
            BalancerJsonRpcNotification::DeregisterAgent => {
                shutdown_tx.send(())?;

                return Ok(ContinuationDecision::Stop);
            }
            BalancerJsonRpcNotification::RegisterAgent(RegisterAgentParams {
                name,
                slots_total,
            }) => {
                let mut agent_controller = AgentController {
                    id: context.agent_id.clone(),
                    name,
                    session,
                    slots_processing: AtomicValue::new(0),
                    slots_total,
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

                Ok(ContinuationDecision::Continue)
            }
            BalancerJsonRpcNotification::UpdateAgentStatus(UpdateAgentStatusParams {
                slots_processing,
            }) => {
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
        }
    }

    async fn on_connection_start(
        _context: Arc<Self::Context>,
        session: &mut Session,
    ) -> Result<ContinuationDecision> {
        if let Err(err) = session
            .text(serde_json::to_string(&AgentJsonRpcNotification::Version(
                VersionParams {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
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
