use std::convert::Infallible;
use std::time::Duration;

use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::Responder;
use actix_web_lab::sse;

use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/agents/stream")]
async fn respond(upstream_peer_pool: web::Data<UpstreamPeerPool>) -> Result<impl Responder, Error> {
    let pool = upstream_peer_pool.clone();

    let event_stream = async_stream::stream! {
        let send_event = |info| {
            match serde_json::to_string(&info) {
                Ok(json) => Some(Ok::<_, Infallible>(sse::Event::Data(sse::Data::new(json)))),
                Err(err) => {
                    eprintln!("Failed to serialize pool info: {err}");
                    None
                }
            }
        };

        if let Some(info) = pool.info() {
            if let Some(event) = send_event(info) {
                yield event;
            }
        }

        loop {
            tokio::select! {
                _ = pool.update_notifier.notified() => {
                    if let Some(info) = pool.info() {
                        if let Some(event) = send_event(info) {
                            yield event;
                        }
                    }
                },
            }
        }
    };

    Ok(sse::Sse::from_stream(event_stream).with_keep_alive(Duration::from_secs(5)))
}
