use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::Responder;

use crate::balancer::supervisor_controller_pool::SupervisorControllerPool;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/supervisors")]
async fn respond(
    supervisor_controller_pool: web::Data<SupervisorControllerPool>,
) -> Result<impl Responder, Error> {
    Ok(web::Json(supervisor_controller_pool.info()))
}
