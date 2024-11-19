use async_trait::async_trait;
use bytes::Bytes;
use log::{error, info};
use pingora_core::upstreams::peer::HttpPeer;
use pingora_core::Result;
use pingora_proxy::{ProxyHttp, Session};
use std::sync::Arc;
use std::time::Duration;

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

    fn response_body_filter(
        &self,
        _session: &mut Session,
        _body: &mut Option<Bytes>,
        end_of_stream: bool,
        ctx: &mut Self::CTX,
    ) -> Result<Option<Duration>>
    where
        Self::CTX: Send + Sync,
    {
        if ctx.uses_slots && end_of_stream {
            // now it's time to restore the used slots
            if let Some(peer) = &ctx.selected_peer {
                match self.upstream_peer_pool.release_slot(&peer.agent_id) {
                    Ok(released) => {
                        if released {
                            if let Err(e) = self.upstream_peer_pool.restore_integrity() {
                                info!("Failed to restore integrity: {}", e);
                            }
                        }
                    }
                    Err(err) => {
                        error!("Failed to release slot: {}", err);
                    }
                }
            }
        }

        Ok(None)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let selected_peer = ctx
            .selected_peer
            .as_ref()
            .expect("Unable to get selected peer");

        let peer = Box::new(HttpPeer::new(
            selected_peer.external_llamacpp_addr,
            false,
            "".to_string(),
        ));

        Ok(peer)
    }
}
