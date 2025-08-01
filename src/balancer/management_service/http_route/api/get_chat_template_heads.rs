use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::Responder;
use actix_web::error::ErrorInternalServerError;

use crate::balancer::state_database::StateDatabase;
use crate::balancer::management_service::http_response::chat_template_heads::ChatTemplateHeads;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/chat_template_heads")]
async fn respond(
    state_database: web::Data<dyn StateDatabase>,
) -> Result<impl Responder, Error> {
    Ok(web::Json(
        ChatTemplateHeads {
            chat_template_heads: state_database
                .list_chat_template_heads()
                .await
                .map_err(ErrorInternalServerError)?,
        }
    ))
}
