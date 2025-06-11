use actix_web::error::JsonPayloadError;
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
    if let Some(info) = upstream_peer_pool.info() {
        Ok(web::Json(info))
    } else {
        Err(JsonPayloadError::Serialize(serde::ser::Error::custom(
            "lock poison error while serializing",
        ))
        .into())
    }
}
