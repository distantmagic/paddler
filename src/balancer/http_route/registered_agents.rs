use actix_web::get;
use actix_web::web;
use actix_web::Error;
use actix_web::Responder;

use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/agents")]
async fn respond(upstream_peer_pool: web::Data<UpstreamPeerPool>) -> Result<impl Responder, Error> {
    Ok(web::Json(upstream_peer_pool))
}
