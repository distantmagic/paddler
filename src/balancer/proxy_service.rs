use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use log::error;
use log::info;
use pingora::http::RequestHeader;
use pingora::proxy::ProxyHttp;
use pingora::proxy::Session;
use pingora::upstreams::peer::HttpPeer;
use pingora::Error;
use pingora::Result;

use crate::balancer::request_context::RequestContext;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

pub struct ProxyService {
    rewrite_host_header: bool,
    check_model: bool,
    slots_endpoint_enable: bool,
    upstream_peer_pool: Arc<UpstreamPeerPool>,
}

impl ProxyService {
    pub fn new(
        rewrite_host_header: bool,
        check_model: bool,
        slots_endpoint_enable: bool,
        upstream_peer_pool: Arc<UpstreamPeerPool>,
    ) -> Self {
        Self {
            rewrite_host_header,
            check_model,
            slots_endpoint_enable,
            upstream_peer_pool,
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
            requested_model: Some("".to_string()),
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
        error!("Error while proxying: {e}");
        if ctx.slot_taken {
            if let Err(err) = ctx.release_slot() {
                error!("Failed to release slot: {err}");

                return Error::new(pingora::InternalError);
            }
        }

        let mut e = e.more_context(format!("Peer: {peer}"));

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
        info!("upstream_peer - {:?} request | rewrite_host_header? {} check_model? {}", session.req_header().method, self.rewrite_host_header, self.check_model);

        // Check if the request method is POST and the content type is JSON
        if self.check_model {
            if session.req_header().method == "POST" {
                // Check if the content type is application/json
                if let Some(content_type) = session.get_header("Content-Type") {
                    if let Ok(content_type_str) = content_type.to_str() {
                        if content_type_str.contains("application/json") {
                            // Read the request body
                            let body = session.read_request_body().await?;
                            if let Some(body) = body {
                                // Parse the JSON payload into a serde_json::Value
                                if let Ok(json_value) = serde_json::from_slice::<serde_json::Value>(&body) {
                                    if let Some(model) = json_value.get("model").and_then(|v| v.as_str()) {
                                        // Set the requested_model field in the RequestContext
                                        ctx.requested_model = Some(model.to_string());
                                        info!("Model in request: {:?}", ctx.requested_model);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let upstream_peer =
            ctx.select_upstream_peer(session.req_header().uri.path(), self.slots_endpoint_enable);

        if ctx.uses_slots && !ctx.slot_taken {
            if let Err(err) = ctx.take_slot() {
                error!("Failed to take slot: {err}");

                return Err(Error::new(pingora::InternalError));
            }
        }

        upstream_peer
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
