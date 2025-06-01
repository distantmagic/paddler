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
use tokio::sync::OwnedSemaphorePermit;

use crate::{
    balancer::{upstream_peer::UpstreamPeer, upstream_peer_pool::UpstreamPeerPool},
    errors::result::Result as PaddlerResult,
};

pub struct LlamaCppContext {
    slot_taken: bool,
    selected_peer: Option<UpstreamPeer>,
    semaphore_permit: Option<OwnedSemaphorePermit>,
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
            semaphore_permit: None,
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

        tokio::select! {
            result = self.upstream_peer_pool.semaphore.clone().acquire_owned() => {
                ctx.semaphore_permit = Some(result.expect("semaphore should never be closed"));
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
