use actix_web::{get, web, Error, HttpResponse, Responder};
use askama_actix::Template;
use log::error;

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

            match template.render() {
                Ok(rendered_template) => Ok(HttpResponse::Ok().body(rendered_template)),
                Err(e) => {
                    error!("Failed to render template: {}", e);
                    return Ok(HttpResponse::InternalServerError().finish());
                }
            }
        }
        Err(e) => {
            error!("Failed to get peers: {}", e);
            Ok(HttpResponse::InternalServerError().finish())
        }
    }
}
