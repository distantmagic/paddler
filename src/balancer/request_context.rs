use std::sync::Arc;

use anyhow::anyhow;
use log::error;
use log::info;
use pingora::Error;
use pingora::Result;

use crate::balancer::upstream_peer::UpstreamPeer;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

pub struct RequestContext {
    pub slot_taken: bool,
    pub selected_peer: Option<UpstreamPeer>,
    pub upstream_peer_pool: Arc<UpstreamPeerPool>,
    pub uses_slots: bool,
    pub requested_model: Option<String>,
}

impl RequestContext {
    pub fn release_slot(&mut self) -> anyhow::Result<()> {
        if let Some(peer) = &self.selected_peer {
            self.upstream_peer_pool
                .release_slot(&peer.agent_id, peer.last_update)?;
            self.upstream_peer_pool.restore_integrity()?;

            self.slot_taken = false;

            Ok(())
        } else {
            Err(anyhow!("There is no peer available to release a slot into"))
        }
    }

    pub fn use_best_peer_and_take_slot(&mut self, model: Option<String>) -> anyhow::Result<Option<UpstreamPeer>> {
        if let Some(peer) = self.upstream_peer_pool.with_agents_write(|agents| {
            let model_str = model.as_deref().unwrap_or("");
            for peer in agents.iter_mut() {
                let is_usable = peer.is_usable();
                let is_usable_for_model = peer.is_usable_for_model(model_str);

                if is_usable && (model.is_none() || is_usable_for_model) {
                    info!("Peer {} is usable: {}, usable for model '{}': {}", peer.agent_id, is_usable, model_str, is_usable_for_model);
                    peer.take_slot()?;
                    return Ok(Some(peer.clone()));
                }
            }
            Ok(None)
        })? {
            self.upstream_peer_pool.restore_integrity()?;

            self.slot_taken = true;

            Ok(Some(peer))
        } else {
            Ok(None)
        }
    }

    pub fn has_peer_supporting_model(&self) -> bool {
        let model_str = self.requested_model.as_deref().unwrap_or("");
        match self.upstream_peer_pool.with_agents_read(|agents| {
            for peer in agents.iter() {
                if peer.supports_model(model_str) {
                    return Ok(true);
                }
            }
            Ok(false)
        }) {
            Ok(result) => result,
            Err(_) => false, // or handle the error as needed
        }
    }

    pub fn select_upstream_peer(&mut self) -> Result<()> {
        let result_option_peer = if self.uses_slots && !self.slot_taken {
            self.use_best_peer_and_take_slot(self.requested_model.clone())
        } else {
            self.upstream_peer_pool.use_best_peer(self.requested_model.clone())
        };

        self.selected_peer = match result_option_peer {
            Ok(peer) => {
                if peer.is_some() {
                    if let Err(e) = self.upstream_peer_pool.restore_integrity() {
                        error!("Failed to restore integrity: {e}");

                        return Err(Error::new(pingora::InternalError));
                    }
                }

                peer
            }
            Err(e) => {
                error!("Failed to get best peer: {e}");

                return Err(Error::new(pingora::InternalError));
            }
        };

        Ok(())
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
            requested_model: Some("llama3".to_string()),
        }
    }

    #[test]
    fn test_take_slot_failure_and_retry() -> anyhow::Result<()> {
        let pool = Arc::new(UpstreamPeerPool::new());
        let mut ctx = create_test_context(pool.clone());

        pool.register_status_update("test_agent", mock_status_update("test_agent", 0, 0))?;

        assert!(ctx.use_best_peer_and_take_slot(ctx.requested_model.clone()).unwrap().is_none());

        assert!(!ctx.slot_taken);
        assert_eq!(ctx.selected_peer, None);

        Ok(())
    }

    #[test]
    fn test_release_slot_success() -> anyhow::Result<()> {
        let pool = Arc::new(UpstreamPeerPool::new());
        let mut ctx = create_test_context(pool.clone());

        pool.register_status_update("test_agent", mock_status_update("test_agent_name", 1, 4))?;

        ctx.select_upstream_peer()?;

        let selected_peer = ctx.selected_peer.clone().unwrap();

        assert_eq!(selected_peer.status.slots_idle, 0);
        assert_eq!(selected_peer.status.slots_processing, 5);

        assert!(ctx.slot_taken);

        ctx.release_slot()?;

        assert!(!ctx.slot_taken);

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.status.slots_idle, 1);
            assert_eq!(peer.status.slots_processing, 4);
            Ok(())
        })?;

        Ok(())
    }

    #[test]
    fn test_release_slot_failure() -> anyhow::Result<()> {
        let pool = Arc::new(UpstreamPeerPool::new());
        let mut ctx = create_test_context(pool.clone());

        pool.register_status_update("test_agent", mock_status_update("test_agent", 5, 0))?;

        assert!(ctx.release_slot().is_err());

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test_agent").unwrap();
            assert_eq!(peer.status.slots_idle, 5);
            assert_eq!(peer.status.slots_processing, 0);
            Ok(())
        })?;

        Ok(())
    }
}
