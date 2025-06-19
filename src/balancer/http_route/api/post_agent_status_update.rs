use actix_web::post;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;
use futures_util::StreamExt as _;
use log::error;
use log::info;
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

struct RemovePeerGuard<'a> {
    pool: &'a UpstreamPeerPool,
    agent_id: String,
}

impl Drop for RemovePeerGuard<'_> {
    fn drop(&mut self) {
        info!("Removing agent: {}", self.agent_id);

        if let Err(err) = self.pool.remove_peer(&self.agent_id) {
            error!("Failed to remove peer: {err}");
        }
    }
}

#[post("/api/v1/agent_status_update/{agent_id}")]
async fn respond(
    path_params: web::Path<PathParams>,
    mut payload: web::Payload,
    upstream_peer_pool: web::Data<UpstreamPeerPool>,
) -> Result<HttpResponse, Error> {
    let _guard = RemovePeerGuard {
        pool: &upstream_peer_pool,
        agent_id: path_params.agent_id.clone(),
    };

    info!("Registering agent: {}", path_params.agent_id);

    while let Some(chunk) = payload.next().await {
        match serde_json::from_slice::<StatusUpdate>(&chunk?) {
            Ok(status_update) => {
                let idle_slots_count = status_update.idle_slots_count;

                if let Err(err) =
                    upstream_peer_pool.register_status_update(&path_params.agent_id, status_update)
                {
                    error!("Failed to register status update: {err}");

                    return Err(Error::from(err));
                }

                if idle_slots_count > 0 {
                    upstream_peer_pool.notifier.notify_one();
                }
            }
            Err(err) => {
                error!("Failed to parse status update: {err}");

                return Err(Error::from(err));
            }
        }
    }

    Ok(HttpResponse::Accepted().finish())
}
