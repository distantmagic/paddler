use actix_web::{post, web, Error, HttpResponse};
use futures_util::StreamExt as _;
use log::error;
use serde::Deserialize;

use crate::balancer::status_update::StatusUpdate;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Deserialize)]
struct PathParams {
    agent_id: String,
}

#[post("/status_update/{agent_id}")]
async fn respond(
    path_params: web::Path<PathParams>,
    mut payload: web::Payload,
    upstream_peers: web::Data<UpstreamPeerPool>,
) -> Result<HttpResponse, Error> {
    while let Some(chunk) = payload.next().await {
        match serde_json::from_slice::<StatusUpdate>(&chunk?) {
            Ok(status_update) => {
                if let Err(e) = upstream_peers.register_status_update(status_update) {
                    error!("Failed to register status update: {}", e);

                    return Err(Error::from(e));
                }
            }
            Err(e) => {
                return Err(Error::from(e));
            }
        }
    }

    upstream_peers.remove_peer(&path_params.agent_id)?;

    Ok(HttpResponse::Accepted().finish())
}
