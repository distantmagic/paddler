use async_trait::async_trait;
use bytes::Bytes;
use log::error;
use pingora::{
    http::RequestHeader,
    protocols::Digest,
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
    Error, ErrorSource, Result,
};
use std::sync::Arc;
use std::time::Duration;

use crate::balancer::upstream_peer::UpstreamPeer;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
use crate::errors::result::Result as PaddlerResult;

// unfortunately pingora does not expose the request internals
// at the moment of writing this there is no wat to get the request path directly
#[inline]
fn strip_host_from_request_summary(request_summary: &str) -> Option<&str> {
    let parts: Vec<&str> = request_summary.split(',').collect();

    return match parts.get(0) {
        Some(part) => Some(part),
        None => None,
    };
}

pub struct LlamaCppContext {
    slot_taken: bool,
    selected_peer: Option<UpstreamPeer>,
    uses_slots: bool,
}

pub struct ProxyService {
    rewrite_host_header: bool,
    slots_endpoint_enable: bool,
    upstream_peer_pool: Arc<UpstreamPeerPool>,
}

impl ProxyService {
    pub fn new(
        rewrite_host_header: bool,
        slots_endpoint_enable: bool,
        upstream_peer_pool: Arc<UpstreamPeerPool>,
    ) -> Self {
        Self {
            rewrite_host_header,
            slots_endpoint_enable,
            upstream_peer_pool,
        }
    }

    #[inline]
    fn release_slot(&self, ctx: &mut LlamaCppContext) -> PaddlerResult<()> {
        if let Some(peer) = &ctx.selected_peer {
            self.upstream_peer_pool
                .release_slot(&peer.agent_id, peer.last_update)?;
            self.upstream_peer_pool.restore_integrity()?;

            ctx.slot_taken = false;
        }

        Ok(())
    }

    #[inline]
    fn take_slot(&self, ctx: &mut LlamaCppContext) -> PaddlerResult<()> {
        if let Some(peer) = &ctx.selected_peer {
            self.upstream_peer_pool.take_slot(&peer.agent_id)?;
            self.upstream_peer_pool.restore_integrity()?;

            ctx.slot_taken = true;
        }

        Ok(())
    }
}

#[async_trait]
impl ProxyHttp for ProxyService {
    type CTX = LlamaCppContext;

    fn new_ctx(&self) -> Self::CTX {
        LlamaCppContext {
            selected_peer: None,
            slot_taken: false,
            uses_slots: false,
        }
    }

    async fn connected_to_upstream(
        &self,
        _session: &mut Session,
        _reused: bool,
        _peer: &HttpPeer,
        #[cfg(unix)] _fd: std::os::unix::io::RawFd,
        #[cfg(windows)] _sock: std::os::windows::io::RawSocket,
        _digest: Option<&Digest>,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if ctx.uses_slots && !ctx.slot_taken {
            if let Err(e) = self.take_slot(ctx) {
                error!("Failed to take slot: {}", e);

                return Err(Error::new(pingora::InternalError));
            }
        }

        Ok(())
    }

    fn error_while_proxy(
        &self,
        peer: &HttpPeer,
        session: &mut Session,
        e: Box<Error>,
        ctx: &mut Self::CTX,
        client_reused: bool,
    ) -> Box<Error> {
        if ctx.slot_taken {
            if let Err(err) = self.release_slot(ctx) {
                error!("Failed to release slot: {}", err);

                return Error::new(pingora::InternalError);
            }
        }

        let mut e = e.more_context(format!("Peer: {}", peer));

        // only reused client connections where retry buffer is not truncated
        e.retry
            .decide_reuse(client_reused && !session.as_ref().retry_buffer_truncated());

        e
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        ctx.uses_slots = match strip_host_from_request_summary(&session.request_summary()) {
            Some("GET /slots") => {
                if !self.slots_endpoint_enable {
                    return Ok(false);
                }

                false
            }
            Some("POST /chat/completions") => true,
            Some("POST /completion") => true,
            Some("POST /v1/chat/completions") => true,
            _ => false,
        };

        ctx.selected_peer = match self.upstream_peer_pool.use_best_peer() {
            Ok(peer) => peer,
            Err(e) => {
                error!("Failed to get best peer: {}", e);

                return Err(Error::new(pingora::InternalError));
            }
        };

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
        if ctx.slot_taken && end_of_stream {
            if let Err(err) = self.release_slot(ctx) {
                error!("Failed to release slot: {}", err);

                return Err(Error::new(pingora::InternalError));
            }
        }

        Ok(None)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        if ctx.selected_peer.is_none() {
            return Err(Error::create(
                pingora::Custom("No peer available"),
                ErrorSource::Upstream,
                None,
                None,
            ));
        }

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

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if self.rewrite_host_header {
            if let Some(peer) = &ctx.selected_peer {
                upstream_request
                    .insert_header("Host".to_string(), peer.external_llamacpp_addr.to_string())?;
            }
        }

        Ok(())
    }
}
