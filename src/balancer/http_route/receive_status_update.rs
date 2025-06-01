use actix_web::{post, web, Error, HttpResponse};
use futures_util::StreamExt as _;
use log::{error, info};
use serde::Deserialize;

use crate::balancer::{status_update::StatusUpdate, upstream_peer_pool::UpstreamPeerPool};

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[derive(Deserialize)]
struct PathParams {
    agent_id: String,
}

struct RemovePeerGuard<'a> {
    pool: &'a UpstreamPeerPool,
    agent_id: String,
}

impl<'a> Drop for RemovePeerGuard<'a> {
    fn drop(&mut self) {
        info!("Removing agent: {}", self.agent_id);

        self.pool.semaphore.forget_permits(1);

        if let Err(e) = self.pool.remove_peer(&self.agent_id) {
            error!("Failed to remove peer: {}", e);
        }
    }
}

#[post("/status_update/{agent_id}")]
async fn respond(
    path_params: web::Path<PathParams>,
    mut payload: web::Payload,
    upstream_peer_pool: web::Data<UpstreamPeerPool>,
) -> Result<HttpResponse, Error> {
    upstream_peer_pool.semaphore.add_permits(1);

    let _guard = RemovePeerGuard {
        pool: &upstream_peer_pool,
        agent_id: path_params.agent_id.clone(),
    };

    info!("Registering agent: {}", path_params.agent_id);

    while let Some(chunk) = payload.next().await {
        match serde_json::from_slice::<StatusUpdate>(&chunk?) {
            Ok(status_update) => {
                if let Err(e) =
                    upstream_peer_pool.register_status_update(&path_params.agent_id, status_update)
                {
                    error!("Failed to register status update: {}", e);

                    return Err(Error::from(e));
                }
            }
            Err(e) => {
                error!("Failed to parse status update: {}", e);

                return Err(Error::from(e));
            }
        }
    }

    Ok(HttpResponse::Accepted().finish())
}
