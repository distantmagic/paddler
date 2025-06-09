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
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    balancer::{upstream_peer::UpstreamPeer, upstream_peer_pool::UpstreamPeerPool},
    errors::result::Result as PaddlerResult,
};

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
            if !self
                .upstream_peer_pool
                .release_slot(&peer.agent_id, peer.last_update)?
            {
                log::warn!(
                    "Failed to release slot for peer {} (Agent ID: {})",
                    peer.external_llamacpp_addr,
                    peer.agent_id
                );
            }
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

    fn error_while_proxy(
        &self,
        peer: &HttpPeer,
        session: &mut Session,
        e: Box<Error>,
        ctx: &mut Self::CTX,
        client_reused: bool,
    ) -> Box<Error> {
        error!("Error while proxying: {}", e);
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

    fn fail_to_connect(
        &self,
        _session: &mut Session,
        _peer: &HttpPeer,
        ctx: &mut Self::CTX,
        mut e: Box<Error>,
    ) -> Box<Error> {
        error!("Failed to connect: {}", e);
        if let Some(peer) = &ctx.selected_peer {
            match self.upstream_peer_pool.quarantine_peer(&peer.agent_id) {
                Ok(true) => {
                    if let Err(err) = self.upstream_peer_pool.restore_integrity() {
                        error!("Failed to restore integrity: {}", err);

                        return Error::new(pingora::InternalError);
                    }

                    // ask server to retry, but try a different best peer
                    ctx.selected_peer = None;
                    e.set_retry(true);
                }
                Ok(false) => {
                    // no need to quarantine for some reason
                }
                Err(err) => {
                    error!("Failed to quarantine peer: {}", err);

                    return Error::new(pingora::InternalError);
                }
            }
        }

        e
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        ctx.uses_slots = match session.req_header().uri.path() {
            "/slots" => {
                if !self.slots_endpoint_enable {
                    return Err(Error::create(
                        pingora::Custom("Slots endpoint is disabled"),
                        ErrorSource::Downstream,
                        None,
                        None,
                    ));
                }

                false
            }
            "/chat/completions" => true,
            "/completion" => true,
            "/v1/chat/completions" => true,
            _ => false,
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
        session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // TODO: Make this customizable.
        const TIMEOUT_SECS: u64 = 30;

        let Some(_req_guard) =
            RequestBufferGuard::increment(&self.upstream_peer_pool.request_buffer_length)
        else {
            session
                .respond_error(pingora::http::StatusCode::TOO_MANY_REQUESTS.as_u16())
                .await?;

            return Err(Error::create(
                pingora::ErrorType::ConnectRefused,
                ErrorSource::Internal,
                None,
                None,
            ));
        };

        let selected_peer = match ctx.selected_peer.clone() {
            Some(p) => p,
            None => {
                tokio::select! {
                    result = async {
                        loop {
                            let result_option_peer = if ctx.uses_slots && !ctx.slot_taken {
                                let rop = self.upstream_peer_pool.use_best_peer_and_take_slot();

                                if let Err(e) = self.upstream_peer_pool.restore_integrity() {
                                    error!("Failed to take slot: {}", e);

                                    return Err(Error::new(pingora::InternalError));
                                }

                                ctx.slot_taken = true;

                                rop
                            } else {
                                self.upstream_peer_pool.use_best_peer()
                            };

                            match result_option_peer {
                                Ok(Some(peer)) => return Ok(peer),
                                Err(e) => {
                                    error!("Failed to get best peer: {}", e);

                                    return Err(Error::new(pingora::InternalError));
                                }
                                // To avoid wasting CPU cycles, we don't immediately retry to
                                // `use_best_peer*` and wait for a notification from code that's
                                // executed when a slot may become available (e.g., the
                                // `/status_update/{agent_id}` endpoint).
                                Ok(None) => self.upstream_peer_pool.notifier.notified().await,
                            }
                        }
                    } => {
                        result?
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_secs(TIMEOUT_SECS)) => {
                        session
                            .respond_error(pingora::http::StatusCode::GATEWAY_TIMEOUT.as_u16())
                            .await?;

                        return Err(Error::create(
                            pingora::ErrorType::ConnectTimedout,
                            ErrorSource::Internal,
                            None,
                            None,
                        ));
                    }
                }
            }
        };

        Ok(Box::new(HttpPeer::new(
            selected_peer.external_llamacpp_addr,
            false,
            "".to_string(),
        )))
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

struct RequestBufferGuard<'a>(&'a AtomicUsize);

impl<'a> RequestBufferGuard<'a> {
    fn increment(length: &'a AtomicUsize) -> Option<Self> {
        // TODO: Make this customizable.
        const REQUEST_BUFFER_CAP: usize = 32;

        if length.load(Ordering::Relaxed) >= REQUEST_BUFFER_CAP {
            None
        } else {
            length.fetch_add(1, Ordering::Relaxed);

            Some(Self(length))
        }
    }
}

impl Drop for RequestBufferGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;

    fn create_test_context() -> LlamaCppContext {
        LlamaCppContext {
            slot_taken: false,
            selected_peer: Some(UpstreamPeer::new(
                "test_agent".to_string(),
                Some("test_name".to_string()),
                None,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
                Some(true),
                Some(true),
                5, // 5 idle slots
                0, // 0 processing slots
            )),
            uses_slots: true,
        }
    }

    #[test]
    fn test_take_slot_success() {
        let pool = Arc::new(UpstreamPeerPool::new());
        let service = ProxyService::new(true, true, pool.clone());
        let mut ctx = create_test_context();

        // Add peer to pool
        pool.with_agents_write(|agents| {
            agents.push(ctx.selected_peer.as_ref().unwrap().clone());
            Ok(())
        })
        .unwrap();

        assert!(service.take_slot(&mut ctx).is_ok());
        assert!(ctx.slot_taken);

        // Verify slot was taken
        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.slots_idle, 4);
            assert_eq!(peer.slots_processing, 1);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_take_slot_failure_and_retry() {
        let pool = Arc::new(UpstreamPeerPool::new());
        let service = ProxyService::new(true, true, pool.clone());
        let mut ctx = create_test_context();

        // Add peer with no slots
        pool.with_agents_write(|agents| {
            let mut peer = ctx.selected_peer.as_ref().unwrap().clone();
            peer.slots_idle = 0;
            agents.push(peer);
            Ok(())
        })
        .unwrap();

        // Add another peer with slots
        pool.with_agents_write(|agents| {
            agents.push(UpstreamPeer::new(
                "test_agent2".to_string(),
                Some("test_name2".to_string()),
                None,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
                Some(true),
                Some(true),
                5, // 5 idle slots
                0, // 0 processing slots
            ));
            Ok(())
        })
        .unwrap();

        assert!(service.take_slot(&mut ctx).is_ok());
        assert!(ctx.slot_taken);
        assert_eq!(ctx.selected_peer.as_ref().unwrap().agent_id, "test_agent2");
    }

    #[test]
    fn test_release_slot_success() {
        let pool = Arc::new(UpstreamPeerPool::new());
        let service = ProxyService::new(true, true, pool.clone());
        let mut ctx = create_test_context();
        ctx.slot_taken = true;

        // Add peer with processing slot
        pool.with_agents_write(|agents| {
            let mut peer = ctx.selected_peer.as_ref().unwrap().clone();
            peer.slots_idle = 4;
            peer.slots_processing = 1;
            agents.push(peer);
            Ok(())
        })
        .unwrap();

        assert!(service.release_slot(&mut ctx).is_ok());
        assert!(!ctx.slot_taken);

        // Verify slot was released
        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.slots_idle, 5);
            assert_eq!(peer.slots_processing, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_release_slot_failure() {
        let pool = Arc::new(UpstreamPeerPool::new());
        let service = ProxyService::new(true, true, pool.clone());
        let mut ctx = create_test_context();
        ctx.slot_taken = true;

        // Add peer with no processing slots
        pool.with_agents_write(|agents| {
            let mut peer = ctx.selected_peer.as_ref().unwrap().clone();
            peer.slots_idle = 5;
            peer.slots_processing = 0;
            agents.push(peer);
            Ok(())
        })
        .unwrap();

        assert!(service.release_slot(&mut ctx).is_ok());
        assert!(!ctx.slot_taken);

        // Verify state is unchanged
        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.slots_idle, 5);
            assert_eq!(peer.slots_processing, 0);
            Ok(())
        })
        .unwrap();
    }
}
