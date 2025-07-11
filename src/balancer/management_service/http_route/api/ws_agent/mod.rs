pub mod jsonrpc;

use actix_web::get;
use actix_web::rt;
use actix_web::web::Data;
use actix_web::web::Path;
use actix_web::web::Payload;
use actix_web::web::ServiceConfig;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_ws::AggregatedMessage;
use actix_ws::Session;
use anyhow::Context;
use anyhow::Result;
use futures_util::StreamExt as _;
use log::debug;
use log::error;
use log::info;
use serde::Deserialize;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;

use self::jsonrpc::notification_params::RegisterAgentParams;
use self::jsonrpc::Notification as BalancerJsonRpcNotification;
use crate::agent::jsonrpc::notification_params::VersionParams;
use crate::agent::jsonrpc::Notification as AgentJsonRpcNotification;
use crate::balancer::agent_controller::AgentController;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::fleet_management_database::FleetManagementDatabase;
use crate::jsonrpc::Error as JsonRpcError;

const MAX_CONTINUATION_SIZE: usize = 100 * 1024;
const PING_INTERVAL: Duration = Duration::from_secs(3);

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

async fn handle_text_message(
    fleet_management_database: Data<dyn FleetManagementDatabase>,
    mut session: Session,
    agent_id: String,
    agent_controller_pool: Data<AgentControllerPool>,
    text: &str,
) -> Result<()> {
    match serde_json::from_str::<BalancerJsonRpcNotification>(text) {
        Ok(BalancerJsonRpcNotification::RegisterAgent(RegisterAgentParams {
            name,
        })) => {
            let mut agent_controller = AgentController {
                id: agent_id.clone(),
                name,
                session,
            };

            if let Some(desired_state) = fleet_management_database.read_desired_state().await? {
                agent_controller
                    .set_desired_state(desired_state)
                    .await
                    .context("Unable to set desired state")?;
            }

            agent_controller_pool
                .register_agent_controller(agent_id, agent_controller)
                .context("Unable to regiagent controller")?;
        }
        Err(
            err @ serde_json::Error {
                ..
            },
        ) if err.is_data() || err.is_syntax() => {
            session
                .text(serde_json::to_string(&JsonRpcError::bad_request(Some(
                    err,
                )))?)
                .await
                .context("JSON-RPC syntax error")?;
        }
        Err(err) => {
            error!("Error handling JSON-RPC request: {err:?}");

            session
                .text(serde_json::to_string(&JsonRpcError::server_error(
                    err.into(),
                ))?)
                .await
                .context("Unexpected JSON-RPC serialization request")?;
        }
    };

    Ok(())
}

async fn send_version(session: &mut Session, version: String) -> Result<()> {
    Ok(session
        .text(serde_json::to_string(&AgentJsonRpcNotification::Version(
            VersionParams {
                version: version.to_string(),
            },
        ))?)
        .await?)
}

#[derive(Deserialize)]
struct PathParams {
    agent_id: String,
}

struct RemoveAgentGuard {
    pool: Data<AgentControllerPool>,
    agent_id: String,
}

impl Drop for RemoveAgentGuard {
    fn drop(&mut self) {
        info!("Remoagent: {}", self.agent_id);

        if let Err(err) = self.pool.remove_agent_controller(&self.agent_id) {
            error!("Failed to reagent: {err}");
        }
    }
}

#[get("/api/v1/agent_socket/{agent_id}")]
async fn respond(
    fleet_management_database: Data<dyn FleetManagementDatabase>,
    path_params: Path<PathParams>,
    payload: Payload,
    req: HttpRequest,
    agent_controller_pool: Data<AgentControllerPool>,
) -> Result<HttpResponse, Error> {
    let agent_id = path_params.agent_id.clone();

    let (res, mut session, msg_stream) = actix_ws::handle(&req, payload)?;

    let mut aggregated_msg_stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(MAX_CONTINUATION_SIZE);

    rt::spawn(async move {
        let _guard = RemoveAgentGuard {
            pool: agent_controller_pool.clone(),
            agent_id: agent_id.clone(),
        };

        if let Err(err) = send_version(&mut session, env!("CARGO_PKG_VERSION").to_string()).await {
            error!("Error sending version: {err:?}");

            return;
        }

        let mut ping_ticker = interval(PING_INTERVAL);

        ping_ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                msg = aggregated_msg_stream.next() => {
                    match msg {
                        Some(Ok(AggregatedMessage::Binary(_))) => {
                            debug!("Received binary message, but only text messages are supported");
                        }
                        Some(Ok(AggregatedMessage::Close(_))) => break,
                        Some(Ok(AggregatedMessage::Ping(msg))) => {
                            if session.pong(&msg).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(AggregatedMessage::Pong(_))) => {
                            // ignore pong messages
                        }
                        Some(Ok(AggregatedMessage::Text(text))) => {
                            if let Err(err) = handle_text_message(
                                fleet_management_database.clone(),
                                session.clone(),
                                agent_id.clone(),
                                agent_controller_pool.clone(),
                                &text
                            ).await.context(format!("Text message: {text}")) {
                                error!("Error handling message: {err:?}");
                            }
                        }
                        Some(Err(err)) => {
                            error!("Error receiving message: {err:?}");
                            break;
                        },
                        None => {
                            break;
                        }
                    }
                }
                _ = ping_ticker.tick() => {
                    if session.ping(b"").await.is_err() {
                        break;
                    }
                }
            }
        }

        let _ = session.close(None).await;
    });

    Ok(res)
}
