use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;
use serde::Deserialize;
use tokio::time::sleep;
use tokio::time::Duration;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::management_service::http_response::chat_template_override::ChatTemplateOverride;

const TIMEOUT: Duration = Duration::from_secs(3);

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Deserialize)]
struct PathParams {
    agent_id: String,
}

#[get("/api/v1/agent/{agent_id}/chat_template_override")]
async fn respond(
    agent_controller_pool: web::Data<AgentControllerPool>,
    params: web::Path<PathParams>,
) -> Result<impl Responder, Error> {
    let agent_controller = match agent_controller_pool.get_agent_controller(&params.agent_id) {
        Some(agent_controller) => agent_controller,
        None => {
            return Ok(HttpResponse::NotFound().finish());
        }
    };

    let mut connection_close_rx = agent_controller.connection_close_rx.resubscribe();

    match agent_controller.get_chat_template_override().await {
        Ok(mut receive_chat_template_override_controller) => {
            tokio::select! {
                _ = connection_close_rx.recv() => Ok(HttpResponse::BadGateway().finish()),
                _ = sleep(TIMEOUT) => Ok(HttpResponse::GatewayTimeout().finish()),
                chat_template_override = receive_chat_template_override_controller.response_rx.recv() => match chat_template_override {
                    Some(chat_template_override) => Ok(HttpResponse::Ok().json(ChatTemplateOverride {
                        chat_template_override,
                    })),
                    None => Ok(HttpResponse::NotFound().finish()),
                },
            }
        }
        Err(err) => Ok(HttpResponse::InternalServerError().body(format!("{err}"))),
    }
}
