use actix_web::error::ErrorInternalServerError;
use actix_web::put;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::balancer_desired_state::BalancerDesiredState;
use crate::balancer::state_database::StateDatabase;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[put("/api/v1/balancer_desired_state")]
async fn respond(
    balancer_desired_state: web::Json<BalancerDesiredState>,
    state_database: web::Data<dyn StateDatabase>,
) -> Result<impl Responder, Error> {
    let balancer_desired_state_inner = balancer_desired_state.into_inner();

    state_database
        .store_balancer_desired_state(&balancer_desired_state_inner)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::NoContent().finish())
}
