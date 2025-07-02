use actix_web::error::JsonPayloadError;
use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::Responder;

use crate::balancer::supervisor_pool::SupervisorPool;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/supervisors")]
async fn respond(supervisor_pool: web::Data<SupervisorPool>) -> Result<impl Responder, Error> {
    Ok(web::Json(supervisor_pool.info()))
}
