use std::sync::Arc;

use log::error;
use pingora::upstreams::peer::HttpPeer;
use pingora::Error;
use pingora::ErrorSource;
use pingora::Result;

use crate::balancer::upstream_peer::UpstreamPeer;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
use crate::errors::result::Result as PaddlerResult;

pub struct RequestContext {
    pub slot_taken: bool,
    pub selected_peer: Option<UpstreamPeer>,
    pub upstream_peer_pool: Arc<UpstreamPeerPool>,
    pub uses_slots: bool,
}

impl RequestContext {
    pub fn release_slot(&mut self) -> PaddlerResult<()> {
        if let Some(peer) = &self.selected_peer {
            self.upstream_peer_pool
                .release_slot(&peer.agent_id, peer.last_update)?;
            self.upstream_peer_pool.restore_integrity()?;

            self.slot_taken = false;

            Ok(())
        } else {
            Err("There is no peer available to release a slot into".into())
        }
    }

    pub fn take_slot(&mut self) -> PaddlerResult<()> {
        if let Some(peer) = &self.selected_peer {
            self.upstream_peer_pool.take_slot(&peer.agent_id)?;
            self.upstream_peer_pool.restore_integrity()?;

            self.slot_taken = true;

            Ok(())
        } else {
            Err("There is no peer available to take a slot from".into())
        }
    }

    pub fn select_upstream_peer(
        &mut self,
        path: &str,
        slots_endpoint_enable: bool,
    ) -> Result<Box<HttpPeer>> {
        if self.selected_peer.is_none() {
            self.selected_peer = match self.upstream_peer_pool.use_best_peer() {
                Ok(peer) => peer,
                Err(err) => {
                    error!("Failed to get best peer: {err}");

                    return Err(Error::new(pingora::InternalError));
                }
            };
        }

        if self.selected_peer.is_some() {
            self.uses_slots = match path {
                "/slots" => {
                    if !slots_endpoint_enable {
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
        }

        let selected_peer = match self.selected_peer.as_ref() {
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
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::balancer::test::mock_status_update;

    fn create_test_context(upstream_peer_pool: Arc<UpstreamPeerPool>) -> RequestContext {
        RequestContext {
            slot_taken: false,
            selected_peer: None,
            upstream_peer_pool,
            uses_slots: true,
        }
    }

    #[test]
    fn test_take_slot_failure_and_retry() -> PaddlerResult<()> {
        let pool = Arc::new(UpstreamPeerPool::new());
        let mut ctx = create_test_context(pool.clone());

        pool.register_status_update("test_agent", mock_status_update("test_agent", 0, 0))?;

        assert!(ctx.take_slot().is_err());

        assert!(!ctx.slot_taken);
        assert_eq!(ctx.selected_peer, None);

        Ok(())
    }

    #[test]
    fn test_release_slot_success() -> PaddlerResult<()> {
        let pool = Arc::new(UpstreamPeerPool::new());
        let mut ctx = create_test_context(pool.clone());

        pool.register_status_update("test_agent", mock_status_update("test_agent", 1, 4))?;

        let peer = ctx.select_upstream_peer("/test", false)?;

        assert_eq!(peer.to_string(), "addr: 127.0.0.1:8080, scheme: HTTP,");

        ctx.take_slot()?;

        assert!(ctx.slot_taken);

        ctx.release_slot()?;

        assert!(!ctx.slot_taken);

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.slots_idle, 1);
            assert_eq!(peer.slots_processing, 4);
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_release_slot_failure() -> PaddlerResult<()> {
        let pool = Arc::new(UpstreamPeerPool::new());
        let mut ctx = create_test_context(pool.clone());

        pool.register_status_update("test_agent", mock_status_update("test_agent", 5, 0))?;

        assert!(ctx.release_slot().is_err());

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.slots_idle, 5);
            assert_eq!(peer.slots_processing, 0);
            Ok(())
        })?;

        Ok(())
    }
}
