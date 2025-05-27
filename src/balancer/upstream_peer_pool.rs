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

    pub fn release_slot(&self, agent_id: &str, last_update: SystemTime) -> Result<bool> {
        self.with_agents_write(|agents| {
            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                if peer.last_update < last_update {
                    // edge case, but no need to update anything anyway
                    return Ok(false);
                }

                peer.release_slot();

                return Ok(true);
            }

            Ok(false)
        })
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
        self.with_agents_write(|agents| {
            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                peer.take_slot();

                Ok(true)
            } else {
                Ok(false)
            }
        })
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
    fn with_agents_read<TCallback, TResult>(&self, cb: TCallback) -> Result<TResult>
    where
        TCallback: FnOnce(&Vec<UpstreamPeer>) -> Result<TResult>,
    {
        match self.agents.read() {
            Ok(agents) => cb(&agents),
            Err(_) => Err("Failed to acquire read lock".into()),
        }
    }

    #[inline]
    fn with_agents_write<TCallback, TResult>(&self, cb: TCallback) -> Result<TResult>
    where
        TCallback: FnOnce(&mut Vec<UpstreamPeer>) -> Result<TResult>,
    {
        match self.agents.write() {
            Ok(mut agents) => cb(&mut agents),
            Err(_) => Err("Failed to acquire write lock".into()),
        }
    }
}
