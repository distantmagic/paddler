use std::sync::Arc;

use actix_web::get;
use actix_web::rt;
use actix_web::web;
use actix_web::web::Payload;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_ws::AggregatedMessage;
use actix_ws::Session;
use anyhow::Result;
use futures_util::StreamExt as _;
use log::debug;
use log::error;
use tokio::sync::Semaphore;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;

use crate::supervisor::jsonrpc::notification_params::VersionParams;
use crate::supervisor::jsonrpc::Notification as JsonRpcNotification;

const MAX_CONCURRENT_HANDLERS_PER_CONNECTION: usize = 10;
const MAX_CONTINUATION_SIZE: usize = 100 * 1024;
const PING_INTERVAL: Duration = Duration::from_secs(3);

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

async fn send_version(session: &mut Session, version: String) -> Result<()> {
    Ok(session
        .text(serde_json::to_string(&JsonRpcNotification::Version(
            VersionParams {
                version: version.to_string(),
            },
        ))?)
        .await?)
}

#[get("/api/v1/supervisor")]
async fn respond(payload: Payload, req: HttpRequest) -> Result<HttpResponse, Error> {
    let (res, mut session, msg_stream) = actix_ws::handle(&req, payload)?;

    let mut aggregated_msg_stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(MAX_CONTINUATION_SIZE);

    let concurrent_handlers_sem = Arc::new(Semaphore::new(MAX_CONCURRENT_HANDLERS_PER_CONNECTION));

    rt::spawn(async move {
        let mut ping_ticker = interval(PING_INTERVAL);

        ping_ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                msg = aggregated_msg_stream.next() => {
                    match msg {
                        Some(Ok(AggregatedMessage::Binary(_))) => {
                            debug!("[system_socket] Received binary message, but only text messages are supported");
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
                            // let handler_collection_clone = handler_collection.clone();
                            // let sem_clone = concurrent_handlers_sem.clone();
                            // let session_clone = session.clone();
                            //
                            // rt::spawn(async move {
                            //     let _permit = match sem_clone.acquire().await {
                            //         Ok(permit) => permit,
                            //         Err(_) => {
                            //             if let Some(err) = handle_too_many_requests(session_clone).await.err() {
                            //                 error!("Error handling too many requests: {err:?}");
                            //             }
                            //
                            //             return;
                            //         },
                            //     };
                            //
                            //     if let Err(err) = handle(
                            //         handler_collection_clone,
                            //         session_clone,
                            //         &text,
                            //     )
                            //     .await
                            //     {
                            //         error!("Error handling message: {err:?}");
                            //     }
                            // });
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
