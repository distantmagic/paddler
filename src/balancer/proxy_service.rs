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
use std::{sync::Arc, time::Duration};

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
            if !self.upstream_peer_pool.release_slot(&peer.agent_id, peer.last_update)? {
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
        let mut initial_peer_id_attempted = None;

        if let Some(peer) = &ctx.selected_peer {
            initial_peer_id_attempted = Some(peer.agent_id.clone());
            if self.upstream_peer_pool.take_slot(&peer.agent_id)? {
                ctx.slot_taken = true;
                return Ok(());
            } else {
                log::warn!(
                    "Failed to take slot for initially selected peer {} (Agent ID: {}). Peer might have no idle slots or take_slot failed.",
                    peer.external_llamacpp_addr,
                    peer.agent_id
                );
            }
        }

        log::info!("Initial take_slot failed or no peer was suitable/selected. Attempting to find and use another best peer.");
        ctx.selected_peer = self.upstream_peer_pool.use_best_peer()?;

        if let Some(new_peer) = &ctx.selected_peer {
            if initial_peer_id_attempted.as_ref() == Some(&new_peer.agent_id) {
                log::warn!(
                    "use_best_peer returned the same peer {} which already failed take_slot. Aborting.",
                    new_peer.external_llamacpp_addr
                );
                return Err("No other available peers with idle slots after initial peer failed".into());
            }
            
            log::info!(
                "Retrying take_slot with new best peer {} (Agent ID: {})",
                new_peer.external_llamacpp_addr,
                new_peer.agent_id
            );
            if self.upstream_peer_pool.take_slot(&new_peer.agent_id)? {
                ctx.slot_taken = true;
                return Ok(());
            } else {
                log::warn!(
                    "Failed to take slot for new best peer {} (Agent ID: {}). Peer might have no idle slots or take_slot failed.",
                    new_peer.external_llamacpp_addr,
                    new_peer.agent_id
                );
                return Err("No available peers with idle slots; new best peer also failed".into());
            }
        } else {
            log::warn!("No best peer available after initial attempt failed.");
            return Err("No available peers found".into());
        }
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
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        if ctx.selected_peer.is_none() {
            ctx.selected_peer = match self.upstream_peer_pool.use_best_peer() {
                Ok(peer) => peer,
                Err(e) => {
                    error!("Failed to get best peer: {}", e);

                    return Err(Error::new(pingora::InternalError));
                }
            };
        }

        let selected_peer = match ctx.selected_peer.as_ref() {
            Some(peer) => peer,
            None => {
                return Err(Error::create(
                    pingora::Custom("No peer available"),
                    ErrorSource::Upstream,
                    None,
                    None,
                ));
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
        }).unwrap();

        assert!(service.take_slot(&mut ctx).is_ok());
        assert!(ctx.slot_taken);

        // Verify slot was taken
        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.slots_idle, 4);
            assert_eq!(peer.slots_processing, 1);
            Ok(())
        }).unwrap();
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
        }).unwrap();

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
        }).unwrap();

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
        }).unwrap();

        assert!(service.release_slot(&mut ctx).is_ok());
        assert!(!ctx.slot_taken);

        // Verify slot was released
        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.slots_idle, 5);
            assert_eq!(peer.slots_processing, 0);
            Ok(())
        }).unwrap();
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
        }).unwrap();

        assert!(service.release_slot(&mut ctx).is_ok());
        assert!(!ctx.slot_taken);

        // Verify state is unchanged
        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.slots_idle, 5);
            assert_eq!(peer.slots_processing, 0);
            Ok(())
        }).unwrap();
    }
}
