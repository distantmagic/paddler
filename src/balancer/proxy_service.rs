use async_trait::async_trait;
use log::info;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_core::Result;
use pingora_proxy::{ProxyHttp, Session};
use std::sync::Arc;

use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

pub struct LlamaCppContext {
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
        LlamaCppContext { uses_slots: false }
    }

    async fn request_filter(&self, _session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        // ctx.beta_user = check_beta_user(session.req_header());
        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        info!("select upstream_peer");

        let peer = Box::new(HttpPeer::new(
            ("127.0.0.1", 8081),
            false,
            "one.one.one.one".to_string(),
        ));
        Ok(peer)
    }
}
