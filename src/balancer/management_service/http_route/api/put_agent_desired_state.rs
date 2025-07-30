use actix_web::error::ErrorInternalServerError;
use actix_web::put;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;

use crate::agent_desired_state::AgentDesiredState;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::state_database::StateDatabase;
use crate::sets_desired_state::SetsDesiredState as _;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[put("/api/v1/agent_desired_state")]
async fn respond(
    agent_controller_pool: web::Data<AgentControllerPool>,
    agent_desired_state: web::Json<AgentDesiredState>,
    state_database: web::Data<dyn StateDatabase>,
) -> Result<impl Responder, Error> {
    let agent_desired_state_inner = agent_desired_state.into_inner();

    state_database
        .store_agent_desired_state(&agent_desired_state_inner)
        .await
        .map_err(ErrorInternalServerError)?;

    agent_controller_pool
        .set_desired_state(agent_desired_state_inner)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::NoContent().finish())
}
