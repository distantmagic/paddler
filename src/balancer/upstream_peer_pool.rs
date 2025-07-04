use std::sync::atomic::AtomicUsize;
use std::sync::RwLock;
use std::time::Duration;
use std::time::SystemTime;

use anyhow::anyhow;
use anyhow::Result;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::Notify;

use crate::balancer::status_update::StatusUpdate;
use crate::balancer::upstream_peer::UpstreamPeer;

#[derive(Serialize, Deserialize)]
pub struct UpstreamPeerPoolInfo {
    pub agents: Vec<UpstreamPeer>,
}

pub struct UpstreamPeerPool {
    pub agents: RwLock<Vec<UpstreamPeer>>,
    pub available_slots_notifier: Notify,
    pub request_buffer_length: AtomicUsize,
    pub update_notifier: Notify,
}

impl UpstreamPeerPool {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(Vec::new()),
            available_slots_notifier: Notify::new(),
            request_buffer_length: AtomicUsize::new(0),
            update_notifier: Notify::new(),
        }
    }

    pub fn info(&self) -> Option<UpstreamPeerPoolInfo> {
        self.agents.read().ok().map(|agents| UpstreamPeerPoolInfo {
            agents: agents.clone(),
        })
    }

    pub fn quarantine_peer(&self, agent_id: &str) -> Result<bool> {
        let notify_waiters = {
            let mut agents = self
                .agents
                .write()
                .expect("Cannot acquire write lock to quarantine peer");

            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                peer.quarantined_until = Some(SystemTime::now() + Duration::from_secs(10));

                true
            } else {
                false
            }
        };

        if notify_waiters {
            self.update_notifier.notify_waiters();
        }

        Ok(notify_waiters)
    }

    pub fn register_status_update(
        &self,
        agent_id: &str,
        status_update: StatusUpdate,
    ) -> Result<()> {
        let has_idle_slots = status_update.slots_idle > 0;

        {
            let mut agents = self
                .agents
                .write()
                .expect("Cannot acquire write lock to register status update");

            if let Some(upstream_peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                upstream_peer.update_status(status_update);
            } else {
                let new_upstream_peer =
                    UpstreamPeer::new_from_status_update(agent_id.to_string(), status_update);

                agents.push(new_upstream_peer);
            }

            agents.sort();
        };

        if has_idle_slots {
            self.available_slots_notifier.notify_waiters();
        }

        self.update_notifier.notify_waiters();

        Ok(())
    }

    pub fn release_slot(&self, agent_id: &str, last_update: SystemTime) -> Result<()> {
        let notify_available_slots = 'check_release_slot: {
            let mut agents = self
                .agents
                .write()
                .expect("Cannot acquire write lock to release slot");

            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                if peer.last_update < last_update {
                    // edge case, but no need to update anything anyway
                    break 'check_release_slot false;
                }

                peer.release_slot()?;

                true
            } else {
                return Err(anyhow!("There is no agent with id: {agent_id}"));
            }
        };

        if notify_available_slots {
            self.available_slots_notifier.notify_waiters();
            self.update_notifier.notify_waiters();
        }

        Ok(())
    }

    pub fn remove_peer(&self, agent_id: &str) -> Result<()> {
        let notify_waiters = {
            let mut agents = self
                .agents
                .write()
                .expect("Cannot acquire write lock to remove peer");

            if let Some(pos) = agents.iter().position(|p| p.agent_id == agent_id) {
                agents.remove(pos);

                true
            } else {
                false
            }
        };

        if notify_waiters {
            self.update_notifier.notify_waiters();
        }

        Ok(())
    }

    pub fn restore_integrity(&self) -> Result<()> {
        {
            let mut agents = self
                .agents
                .write()
                .expect("Cannot acquire write lock to restore integrity");

            agents.sort();
        };

        self.update_notifier.notify_waiters();

        Ok(())
    }

    #[cfg(feature = "statsd_reporter")]
    /// Returns (slots_idle, slots_processing) tuple.
    pub fn total_slots(&self) -> Result<(usize, usize)> {
        let agents = self
            .agents
            .read()
            .expect("Cannot acquire read lock to get total slots");

        let mut slots_idle = 0;
        let mut slots_processing = 0;

        for peer in agents.iter() {
            slots_idle += peer.status.slots_idle;
            slots_processing += peer.status.slots_processing;
        }

        Ok((slots_idle, slots_processing))
    }

    pub fn use_best_peer(&self) -> Result<Option<UpstreamPeer>> {
        let agents = self
            .agents
            .read()
            .expect("Cannot acquire read lock to use best peer");

        for peer in agents.iter() {
            if peer.is_usable() {
                return Ok(Some(peer.clone()));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::balancer::test::mock_status_update;

    #[test]
    fn test_race_condition_handling() -> Result<()> {
        let pool = UpstreamPeerPool::new();

        pool.register_status_update("test1", mock_status_update("test1", 5, 0))?;

        {
            let mut agents = pool.agents.write().unwrap();

            agents
                .iter_mut()
                .find(|p| p.agent_id == "test1")
                .unwrap()
                .take_slot();
        }

        let last_update_at_selection_time = {
            let agents = pool.agents.read().unwrap();

            agents
                .iter()
                .find(|p| p.agent_id == "test1")
                .unwrap()
                .last_update
        };

        {
            let agents = pool.agents.read().unwrap();
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();

            assert_eq!(peer.slots_taken, 1);
            assert_eq!(peer.status.slots_idle, 4);
            assert_eq!(peer.status.slots_processing, 1);
        }

        pool.register_status_update("test1", mock_status_update("test1", 0, 0))?;

        {
            let agents = pool.agents.read().unwrap();
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();

            assert_eq!(peer.slots_taken, 1);
            assert_eq!(peer.status.slots_idle, 0);
            assert_eq!(peer.status.slots_processing, 0);
        }

        pool.release_slot("test1", last_update_at_selection_time)?;

        {
            let agents = pool.agents.read().unwrap();
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();

            assert_eq!(peer.slots_taken, 0);
            assert_eq!(peer.status.slots_idle, 0);
            assert_eq!(peer.status.slots_processing, 0);
        }

        Ok(())
    }

    #[test]
    fn test_use_best_peer() -> Result<()> {
        let pool = UpstreamPeerPool::new();

        pool.register_status_update("test1", mock_status_update("test1", 5, 0))?;
        pool.register_status_update("test2", mock_status_update("test2", 3, 0))?;
        pool.register_status_update("test3", mock_status_update("test3", 0, 0))?;

        let best_peer = pool.use_best_peer()?.unwrap();

        assert_eq!(best_peer.agent_id, "test1");
        assert_eq!(best_peer.status.slots_idle, 5);

        Ok(())
    }
}
