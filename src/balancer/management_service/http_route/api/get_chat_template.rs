use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Error;
use actix_web::Responder;
use actix_web::error::ErrorInternalServerError;
use serde::Deserialize;

use crate::balancer::state_database::StateDatabase;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Deserialize)]
struct PathParams {
    chat_template_id: String,
}

#[get("/api/v1/chat_template/{chat_template_id}")]
async fn respond(
    path: web::Path<PathParams>,
    state_database: web::Data<dyn StateDatabase>,
) -> Result<impl Responder, Error> {
    let chat_template = state_database
        .read_chat_template(path.chat_template_id.clone())
        .await
        .map_err(ErrorInternalServerError)?;

    if chat_template.is_none() {
        Ok(HttpResponse::NotFound().finish())
    } else {
        Ok(HttpResponse::Ok().json(chat_template))
    }
}
