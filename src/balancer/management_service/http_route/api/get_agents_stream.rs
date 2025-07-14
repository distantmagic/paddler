use std::convert::Infallible;
use std::time::Duration;

use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::Responder;
use actix_web_lab::sse;
use log::error;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::produces_snapshot::ProducesSnapshot as _;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/agents/stream")]
async fn respond(
    agent_controller_pool: web::Data<AgentControllerPool>,
) -> Result<impl Responder, Error> {
    let pool = agent_controller_pool.clone();

    let event_stream = async_stream::stream! {
        let send_event = |info| {
            match serde_json::to_string(&info) {
                Ok(json) => Some(Ok::<_, Infallible>(sse::Event::Data(sse::Data::new(json)))),
                Err(err) => {
                    error!("Failed to serialize pool info: {err}");
                    None
                }
            }
        };

        if let Some(event) = send_event(pool.make_snapshot()) {
            yield event;
        }

        loop {
            tokio::select! {
                _ = pool.update_notifier.notified() => {
                    if let Some(event) = send_event(pool.make_snapshot()) {
                        yield event;
                    }
                },
            }
        }
    };

    Ok(sse::Sse::from_stream(event_stream).with_keep_alive(Duration::from_secs(10)))
}
