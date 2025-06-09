use serde::Serialize;
use std::{
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::{
    balancer::{status_update::StatusUpdate, upstream_peer::{UpstreamPeer, UpstreamPeerInfo}},
    errors::result::Result,
};

#[derive(Serialize)]
pub struct UpstreamPeerPool {
    pub agents: RwLock<Vec<UpstreamPeer>>,
    #[serde(skip_serializing)]
    pub upstream_slots_permits: Arc<Semaphore>,
}

impl Clone for UpstreamPeerPool {
    fn clone(&self) -> Self {
        let agents = self
            .with_agents_read(|agents| Ok(agents.clone()))
            .unwrap_or_default();

        UpstreamPeerPool {
            agents: RwLock::new(agents),
        }
    }
}

impl UpstreamPeerPool {
    pub fn new() -> Self {
        UpstreamPeerPool {
            agents: RwLock::new(Vec::new()),
            upstream_slots_permits: Arc::new(Semaphore::new(0)),
        }
    }

    pub fn quarantine_peer(&self, agent_id: &str) -> Result<bool> {
        self.with_agents_write(|agents| {
            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                peer.quarantined_until = Some(SystemTime::now() + Duration::from_secs(10));

                return Ok(true);
            }

            Ok(false)
        })
    }

    pub fn register_status_update(
        &self,
        agent_id: &str,
        status_update: StatusUpdate,
    ) -> Result<()> {
        self.with_agents_write(|agents| {
            if let Some(upstream_peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                let update_slots_count = status_update.idle_slots_count + status_update.processing_slots_count;
                if update_slots_count > upstream_peer.slots_count() {
                    let delta = update_slots_count  - upstream_peer.slots_count();
                    self.upstream_slots_permits.add_permits(delta);
                }

                if upstream_peer.slots_idle > status_update.idle_slots_count {
                    let delta = upstream_peer.slots_idle - status_update.idle_slots_count;
                    self.upstream_slots_permits.forget_permits(delta);
                }

                upstream_peer.update_status(status_update);
            } else {
                let new_upstream_peer =
                    UpstreamPeer::new_from_status_update(agent_id.to_string(), status_update);
                self.upstream_slots_permits.add_permits(new_upstream_peer.slots_count());
                agents.push(new_upstream_peer);
            }

            agents.sort();

            Ok(())
        })
    }

    pub fn release_slot(
        &self,
        agent_id: &str,
        last_update_from_context: SystemTime,
    ) -> Result<bool> {
        let mut needs_integrity_restore = false;
        let successful_release = self.with_agents_write(|agents| {
            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                if peer.last_update > last_update_from_context {
                    log::warn!(
                        "Peer {} (Agent ID: {}): Was updated after selection (pool_time: {:?}, selection_time: {:?}). Skipping relative slot release, trusting agent update.",
                        peer.external_llamacpp_addr,
                        peer.agent_id,
                        peer.last_update,
                        last_update_from_context
                    );
                    Ok(false)
                } else {
                    if peer.release_slot() {
                        Ok(true)
                    } else {
                        log::warn!(
                            "Peer {} (Agent ID: {}): peer.release_slot() returned false. State might be inconsistent (e.g. slots_processing was 0).",
                            peer.external_llamacpp_addr,
                            peer.agent_id
                        );
                        needs_integrity_restore = true;
                        Ok(false)
                    }
                }
            } else {
                Ok(false)
            }
        })?;

        if needs_integrity_restore {
            self.restore_integrity()?;
        }
        Ok(successful_release)
    }

    pub fn remove_peer(&self, agent_id: &str) -> Result<()> {
        self.with_agents_write(|agents| {
            if let Some(pos) = agents.iter().position(|p| p.agent_id == agent_id) {
                let slots_count = agents[pos].slots_count();
                agents.remove(pos);
                self.upstream_slots_permits.forget_permits(slots_count);
            }
            Ok(())
        })
    }

    pub fn restore_integrity(&self) -> Result<()> {
        self.with_agents_write(|agents| {
            agents.sort();

            Ok(())
        })
    }

    pub fn take_slot(&self, agent_id: &str) -> Result<bool> {
        let mut needs_integrity_restore = false;
        let successful_take = self.with_agents_write(|agents| {
            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                if peer.take_slot() {
                    Ok(true)
                } else {
                    log::warn!(
                        "Peer {} (Agent ID: {}): peer.take_slot() returned false. Peer might have no idle slots.",
                        peer.external_llamacpp_addr,
                        peer.agent_id
                    );
                    needs_integrity_restore = true;
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        })?;

        if needs_integrity_restore {
            self.restore_integrity()?;
        }
        Ok(successful_take)
    }

    pub fn store_permit(&self, agent_id: &str, permit: OwnedSemaphorePermit) -> Result<bool> {
        self.with_agents_write(|agents| {
            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                peer.store_permit(permit);
                Ok(true)
            } else {
                Ok(false)
            }
        })
    }

    pub fn release_one_permit(&self, agent_id: &str) -> Result<()> {
        self.with_agents_write(|agents| {
            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                peer.release_permits(1);
            }
            Ok(())
        })
    }

    #[cfg(feature = "statsd_reporter")]
    // returns (slots_idle, slots_processing) tuple
    pub fn total_slots(&self) -> Result<(usize, usize)> {
        self.with_agents_read(|agents| {
            let mut slots_idle = 0;
            let mut slots_processing = 0;

            for peer in agents.iter() {
                slots_idle += peer.slots_idle;
                slots_processing += peer.slots_processing;
            }

            Ok((slots_idle, slots_processing))
        })
    }

    pub fn use_best_peer(&self) -> Result<Option<UpstreamPeerInfo>> {
        self.with_agents_write(|agents| {
            for peer in agents.iter_mut() {
                if peer.is_usable() {
                    return Ok(Some(peer.info()));
                }
            }

            return Ok(None);
        })
    }

    #[cfg(feature = "statsd_reporter")]
    #[inline]
    pub fn with_agents_read<TCallback, TResult>(&self, cb: TCallback) -> Result<TResult>
    where
        TCallback: FnOnce(&Vec<UpstreamPeer>) -> Result<TResult>,
    {
        match self.agents.read() {
            Ok(agents) => cb(&agents),
            Err(_) => Err("Failed to acquire read lock".into()),
        }
    }

    #[inline]
    pub fn with_agents_write<TCallback, TResult>(&self, cb: TCallback) -> Result<TResult>
    where
        TCallback: FnOnce(&mut Vec<UpstreamPeer>) -> Result<TResult>,
    {
        match self.agents.write() {
            Ok(mut agents) => cb(&mut agents),
            Err(_) => Err("Failed to acquire write lock".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn create_test_peer(id: &str, idle: usize, processing: usize) -> UpstreamPeer {
        UpstreamPeer::new(
            id.to_string(),
            Some(format!("test_{}", id)),
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            Some(true),
            Some(true),
            idle,
            processing,
        )
    }

    #[test]
    fn test_take_slot_success() {
        let pool = UpstreamPeerPool::new();
        let peer = create_test_peer("test1", 5, 0);

        pool.with_agents_write(|agents| {
            agents.push(peer);
            Ok(())
        })
        .unwrap();

        assert!(pool.take_slot("test1").unwrap());

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(peer.slots_idle, 4);
            assert_eq!(peer.slots_processing, 1);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_take_slot_prevents_underflow() {
        let pool = UpstreamPeerPool::new();
        let peer = create_test_peer("test1", 0, 0); // No idle slots
        pool.with_agents_write(|agents| {
            agents.push(peer);
            Ok(())
        })
        .unwrap();
        // This should fail gracefully, not underflow
        assert!(!pool.take_slot("test1").unwrap());
        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(peer.slots_idle, 0); // Should remain 0
            assert_eq!(peer.slots_processing, 0); // Should remain 0
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_take_slot_failure() {
        let pool = UpstreamPeerPool::new();
        let peer = create_test_peer("test1", 0, 0);

        pool.with_agents_write(|agents| {
            agents.push(peer);
            Ok(())
        })
        .unwrap();

        assert!(!pool.take_slot("test1").unwrap());

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(peer.slots_idle, 0);
            assert_eq!(peer.slots_processing, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_release_slot_success() {
        let pool = UpstreamPeerPool::new();
        let peer = create_test_peer("test1", 4, 1);

        pool.with_agents_write(|agents| {
            agents.push(peer);
            Ok(())
        })
        .unwrap();

        assert!(pool.take_slot("test1").unwrap());

        let last_update_at_selection_time = pool
            .with_agents_read(|agents| {
                Ok(agents
                    .iter()
                    .find(|p| p.agent_id == "test1")
                    .unwrap()
                    .last_update)
            })
            .unwrap();

        assert!(pool
            .release_slot("test1", last_update_at_selection_time)
            .unwrap());

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(
                peer.slots_idle, 4,
                "Idle slots should be 4 after take and release"
            );
            assert_eq!(
                peer.slots_processing, 1,
                "Processing slots should be 1 after take and release"
            );
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_release_slot_prevents_underflow() {
        let pool = UpstreamPeerPool::new();
        let peer = create_test_peer("test1", 5, 0); // No processing slots
        pool.with_agents_write(|agents| {
            agents.push(peer);
            Ok(())
        })
        .unwrap();
        // This should fail gracefully, not underflow
        assert!(!pool.release_slot("test1", SystemTime::now()).unwrap());
        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(peer.slots_idle, 5); // Should remain 5
            assert_eq!(peer.slots_processing, 0); // Should remain 0
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_release_slot_failure() {
        let pool = UpstreamPeerPool::new();
        let peer = create_test_peer("test1", 5, 0);

        pool.with_agents_write(|agents| {
            agents.push(peer);
            Ok(())
        })
        .unwrap();

        assert!(!pool.release_slot("test1", SystemTime::now()).unwrap());

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(peer.slots_idle, 5);
            assert_eq!(peer.slots_processing, 0);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_race_condition_handling() {
        let pool = UpstreamPeerPool::new();
        let peer = create_test_peer("test1", 5, 0);

        pool.with_agents_write(|agents| {
            agents.push(peer);
            Ok(())
        })
        .unwrap();

        assert!(pool.take_slot("test1").unwrap());

        let last_update_at_selection_time = pool
            .with_agents_read(|agents| {
                Ok(agents
                    .iter()
                    .find(|p| p.agent_id == "test1")
                    .unwrap()
                    .last_update)
            })
            .unwrap();

        let status_update = StatusUpdate::new(
            Some("test1".to_string()),
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            Some(true),
            Some(true),
            vec![],
        );
        pool.register_status_update("test1", status_update).unwrap();

        assert!(!pool
            .release_slot("test1", last_update_at_selection_time)
            .unwrap());

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(
                peer.slots_idle, 0,
                "Idle slots should be 0 after status update"
            );
            assert_eq!(
                peer.slots_processing, 0,
                "Processing slots should be 0 after status update"
            );
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn test_use_best_peer() {
        let pool = UpstreamPeerPool::new();

        pool.with_agents_write(|agents| {
            agents.push(create_test_peer("test1", 5, 0));
            agents.push(create_test_peer("test2", 3, 0));
            agents.push(create_test_peer("test3", 0, 0));
            Ok(())
        })
        .unwrap();

        let best_peer = pool.use_best_peer().unwrap().unwrap();
        assert_eq!(best_peer.agent_id, "test1");
        assert_eq!(best_peer.slots_idle, 5);
    }
}
