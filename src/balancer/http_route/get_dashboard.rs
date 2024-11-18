use actix_web::{get, web, Error, HttpResponse, Responder};
use askama_actix::Template;
use futures_util::StreamExt as _;
use log::error;
use serde::Deserialize;

use crate::balancer::status_update::StatusUpdate;
use crate::balancer::upstream_peer::UpstreamPeer;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    peers: Vec<UpstreamPeer>,
}

#[get("/dashboard")]
async fn respond(upstream_peer_pool: web::Data<UpstreamPeerPool>) -> Result<impl Responder, Error> {
    match upstream_peer_pool.get_cloned_peers() {
        Ok(peers) => {
            let template = DashboardTemplate { peers };

            Ok(HttpResponse::Ok().body(template.render().unwrap()))
        }
        Err(e) => {
            error!("Failed to get peers: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
