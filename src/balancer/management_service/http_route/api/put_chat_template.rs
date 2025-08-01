use actix_web::put;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Error;
use actix_web::Responder;
use actix_web::error::ErrorInternalServerError;

use crate::chat_template::ChatTemplate;
use crate::balancer::state_database::StateDatabase;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[put("/api/v1/chat_template")]
async fn respond(
    chat_template: web::Json<ChatTemplate>,
    state_database: web::Data<dyn StateDatabase>,
) -> Result<impl Responder, Error> {
    state_database
        .store_chat_template(&chat_template.into_inner())
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::NoContent().finish())
}
