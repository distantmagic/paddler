use actix_web::error::ErrorInternalServerError;
use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::balancer::state_database::StateDatabase;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/agent_desired_state")]
async fn respond(state_database: web::Data<dyn StateDatabase>) -> Result<impl Responder, Error> {
    let desired_state = state_database
        .read_desired_state()
        .await
        .map_err(ErrorInternalServerError)?
        .unwrap_or_default();

    Ok(HttpResponse::Ok().json(desired_state))
}
