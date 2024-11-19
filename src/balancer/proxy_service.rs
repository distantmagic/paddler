use async_trait::async_trait;
use log::info;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_core::Result;
use pingora_proxy::{ProxyHttp, Session};
use std::sync::Arc;

use crate::balancer::upstream_peer::UpstreamPeer;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

// unfortunately pingora does not expose the request internals
// at the moment of writing this there is no wat to get the request path directly
fn strip_host_from_request_summary(request_summary: &str) -> Option<&str> {
    let parts: Vec<&str> = request_summary.split(',').collect();

    return match parts.get(0) {
        Some(part) => Some(part),
        None => None,
    };
}

pub struct LlamaCppContext {
    selected_peer: Option<UpstreamPeer>,
    uses_slots: bool,
}

pub struct ProxyService {
    upstream_peer_pool: Arc<UpstreamPeerPool>,
}

impl ProxyService {
    pub fn new(upstream_peer_pool: Arc<UpstreamPeerPool>) -> Self {
        Self { upstream_peer_pool }
    }
}

#[async_trait]
impl ProxyHttp for ProxyService {
    type CTX = LlamaCppContext;

    fn new_ctx(&self) -> Self::CTX {
        LlamaCppContext {
            selected_peer: None,
            uses_slots: false,
        }
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        ctx.uses_slots = match strip_host_from_request_summary(&session.request_summary()) {
            Some("POST /chat/completions") => true,
            Some("POST /completion") => true,
            Some("POST /v1/chat/completions") => true,
            _ => false,
        };

        ctx.selected_peer = match self.upstream_peer_pool.use_best_peer(ctx.uses_slots) {
            Ok(peer) => peer,
            Err(e) => {
                info!("Failed to get best peer: {}", e);

                return Ok(true);
            }
        };

        if ctx.selected_peer.is_some() && ctx.uses_slots {
            if let Err(e) = self.upstream_peer_pool.restore_integrity() {
                info!("Failed to restore integrity: {}", e);

                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let selected_peer = ctx
            .selected_peer
            .as_ref()
            .expect("Selected peer is not set");

        let peer = Box::new(HttpPeer::new(
            selected_peer.external_llamacpp_addr,
            false,
            "".to_string(),
        ));

        Ok(peer)
    }
}
