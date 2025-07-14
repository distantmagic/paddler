use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::Responder;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::produces_snapshot::ProducesSnapshot as _;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/agents")]
async fn respond(
    agent_controller_pool: web::Data<AgentControllerPool>,
) -> Result<impl Responder, Error> {
    Ok(web::Json(agent_controller_pool.make_snapshot()))
}
