use std::convert::Infallible;
use std::time::Duration;

use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::Responder;
use actix_web_lab::sse;
use log::error;

use crate::balancer::management_service::http_response::chat_template_heads::ChatTemplateHeads;
use crate::balancer::state_database::StateDatabase;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/chat_template_heads/stream")]
async fn respond(
    state_database: web::Data<dyn StateDatabase>,
) -> Result<impl Responder, Error> {
    let update_notifier = state_database.get_update_notifier();
    let event_stream = async_stream::stream! {
        let send_event = |info| {
            match serde_json::to_string(&info) {
                Ok(json) => Some(Ok::<_, Infallible>(sse::Event::Data(sse::Data::new(json)))),
                Err(err) => {
                    error!("Failed to serialize buffered requests info: {err}");
                    None
                }
            }
        };

        loop {
            match state_database.list_chat_template_heads().await {
                Ok(info) => {
                    if let Some(event) = send_event(ChatTemplateHeads {
                        chat_template_heads: info,
                    }) {
                        yield event;
                    }
                },
                Err(err) => {
                    error!("Failed to list chat template heads: {err}");
                }
            };

            update_notifier.notified().await;
        }
    };

    Ok(sse::Sse::from_stream(event_stream).with_keep_alive(Duration::from_secs(10)))
}
