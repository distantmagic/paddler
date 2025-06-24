use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use log::error;
use pingora::http::RequestHeader;
use pingora::proxy::ProxyHttp;
use pingora::proxy::Session;
use pingora::upstreams::peer::HttpPeer;
use pingora::Error;
use pingora::ErrorSource;
use pingora::Result;

use crate::balancer::request_context::RequestContext;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

struct RequestBufferGuard<'a>(&'a AtomicUsize);

impl<'a> RequestBufferGuard<'a> {
    fn increment(length: &'a AtomicUsize, max_buffered_requests: usize) -> Option<Self> {
        if length.load(Ordering::Relaxed) >= max_buffered_requests {
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

pub struct ProxyService {
    buffered_request_timeout: Duration,
    max_buffered_requests: usize,
    rewrite_host_header: bool,
    slots_endpoint_enable: bool,
    upstream_peer_pool: Arc<UpstreamPeerPool>,
}

impl ProxyService {
    pub fn new(
        rewrite_host_header: bool,
        slots_endpoint_enable: bool,
        upstream_peer_pool: Arc<UpstreamPeerPool>,
        buffered_request_timeout: Duration,
        max_buffered_requests: usize,
    ) -> Self {
        Self {
            rewrite_host_header,
            slots_endpoint_enable,
            upstream_peer_pool,
            buffered_request_timeout,
            max_buffered_requests,
        }
    }
}

#[async_trait]
impl ProxyHttp for ProxyService {
    type CTX = RequestContext;

    fn new_ctx(&self) -> Self::CTX {
        RequestContext {
            selected_peer: None,
            slot_taken: false,
            upstream_peer_pool: self.upstream_peer_pool.clone(),
            uses_slots: false,
        }
    }

    fn error_while_proxy(
        &self,
        peer: &HttpPeer,
        session: &mut Session,
        proxy_error: Box<Error>,
        ctx: &mut Self::CTX,
        client_reused: bool,
    ) -> Box<Error> {
        error!("Error while proxying: {proxy_error}");

        if ctx.slot_taken {
            if let Err(err) = ctx.release_slot() {
                error!("Failed to release slot: {err}");

                return Error::new(pingora::InternalError);
            }
        }

        let mut proxy_error_with_context = proxy_error.more_context(format!("Peer: {peer}"));

        // only reused client connections where retry buffer is not truncated
        proxy_error_with_context
            .retry
            .decide_reuse(client_reused && !session.as_ref().retry_buffer_truncated());

        proxy_error_with_context
    }

    fn fail_to_connect(
        &self,
        _session: &mut Session,
        _peer: &HttpPeer,
        ctx: &mut Self::CTX,
        mut connection_err: Box<Error>,
    ) -> Box<Error> {
        error!("Failed to connect: {connection_err}");

        if let Some(peer) = &ctx.selected_peer {
            match self.upstream_peer_pool.quarantine_peer(&peer.agent_id) {
                Ok(true) => {
                    if let Err(err) = self.upstream_peer_pool.restore_integrity() {
                        error!("Failed to restore integrity: {err}");

                        return Error::new(pingora::InternalError);
                    }

                    // ask server to retry, but try a different best peer
                    ctx.selected_peer = None;
                    connection_err.set_retry(true);
                }
                Ok(false) => {
                    // no need to quarantine for some reason
                }
                Err(err) => {
                    error!("Failed to quarantine peer: {err}");

                    return Error::new(pingora::InternalError);
                }
            }
        }

        connection_err
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
            if let Err(err) = ctx.release_slot() {
                error!("Failed to release slot: {err}");

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

        let peer = tokio::select! {
            result = async {
                loop {
                    ctx.select_upstream_peer()?;

                    if let Some(peer) = ctx.selected_peer.clone() {
                        return Ok::<_, Box<Error>>(peer)
                    }

                    let Some(_req_guard) = RequestBufferGuard::increment(
                        &self.upstream_peer_pool.request_buffer_length,
                        self.max_buffered_requests,
                    ) else {
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

                    // To avoid wasting CPU cycles, we don't immediately retry to
                    // `select_upstream_peer` and wait for a notification from code that's
                    // executed when a slot may become available (e.g., the
                    // `/api/v1/agent_status_update/{agent_id}` endpoint).
                    self.upstream_peer_pool.available_slots_notifier.notified().await;
                }
            } => {
                result?
            }
            _ = tokio::time::sleep(self.buffered_request_timeout) => {
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
        };

        Ok(HttpPeer::new(peer.status.external_llamacpp_addr, false, "".into()).into())
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if self.rewrite_host_header {
            if let Some(peer) = &ctx.selected_peer {
                upstream_request.insert_header(
                    "Host".to_string(),
                    peer.status.external_llamacpp_addr.to_string(),
                )?;
            }
        }

        Ok(())
    }
}
