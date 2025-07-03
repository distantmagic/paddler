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
use anyhow::Result;
use futures_util::StreamExt as _;
use log::debug;
use log::error;
use log::info;
use serde::Deserialize;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;

use self::jsonrpc::notification_params::RegisterSupervisorParams;
use self::jsonrpc::Notification as BalancerJsonRpcNotification;
use crate::balancer::supervisor_controller::SupervisorController;
use crate::balancer::supervisor_controller_pool::SupervisorControllerPool;
use crate::supervisor::jsonrpc::notification_params::VersionParams;
use crate::supervisor::jsonrpc::Notification as SupervisorJsonRpcNotification;

const MAX_CONTINUATION_SIZE: usize = 100 * 1024;
const PING_INTERVAL: Duration = Duration::from_secs(3);

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

async fn handle_text_message(
    mut session: Session,
    supervisor_id: String,
    supervisor_controller_pool: Data<SupervisorControllerPool>,
    text: &str,
) -> Result<()> {
    match serde_json::from_str::<BalancerJsonRpcNotification>(text) {
        Ok(BalancerJsonRpcNotification::RegisterSupervisor(RegisterSupervisorParams {
            name,
        })) => {
            supervisor_controller_pool.register_supervisor_controller(
                supervisor_id.clone(),
                SupervisorController {
                    id: supervisor_id,
                    name,
                    session,
                },
            )?;
        }
        Err(
            err @ serde_json::Error {
                ..
            },
        ) if err.is_data() || err.is_syntax() => {
            session
                .text(serde_json::to_string(
                    &SupervisorJsonRpcNotification::bad_request(Some(err)),
                )?)
                .await?;
        }
        Err(err) => {
            error!("Error handling JSON-RPC request: {err:?}");

            session
                .text(serde_json::to_string(
                    &SupervisorJsonRpcNotification::bad_request(None),
                )?)
                .await?;
        }
    };

    Ok(())
}

async fn send_version(session: &mut Session, version: String) -> Result<()> {
    Ok(session
        .text(serde_json::to_string(
            &SupervisorJsonRpcNotification::Version(VersionParams {
                version: version.to_string(),
            }),
        )?)
        .await?)
}

#[derive(Deserialize)]
struct PathParams {
    supervisor_id: String,
}

struct RemoveSupervisorGuard {
    pool: Data<SupervisorControllerPool>,
    supervisor_id: String,
}

impl Drop for RemoveSupervisorGuard {
    fn drop(&mut self) {
        info!("Removing supervisor: {}", self.supervisor_id);

        if let Err(err) = self.pool.remove_supervisor_controller(&self.supervisor_id) {
            error!("Failed to remove supervisor: {err}");
        }
    }
}

#[get("/api/v1/supervisor_socket/{supervisor_id}")]
async fn respond(
    path_params: Path<PathParams>,
    payload: Payload,
    req: HttpRequest,
    supervisor_controller_pool: Data<SupervisorControllerPool>,
) -> Result<HttpResponse, Error> {
    let supervisor_id = path_params.supervisor_id.clone();
    let _guard = RemoveSupervisorGuard {
        pool: supervisor_controller_pool.clone(),
        supervisor_id: supervisor_id.clone(),
    };

    let (res, mut session, msg_stream) = actix_ws::handle(&req, payload)?;

    let mut aggregated_msg_stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(MAX_CONTINUATION_SIZE);

    rt::spawn(async move {
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
                                session.clone(),
                                supervisor_id.clone(),
                                supervisor_controller_pool.clone(),
                                &text
                            ).await {
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
