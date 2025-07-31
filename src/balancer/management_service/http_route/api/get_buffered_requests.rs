use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::Responder;

use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::produces_snapshot::ProducesSnapshot as _;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/buffered_requests")]
async fn respond(
    buffered_request_manager: web::Data<BufferedRequestManager>,
) -> Result<impl Responder, Error> {
    Ok(web::Json(buffered_request_manager.make_snapshot()))
}
