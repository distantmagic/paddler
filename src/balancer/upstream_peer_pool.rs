use std::sync::atomic::AtomicUsize;
use std::sync::RwLock;
use std::time::Duration;
use std::time::SystemTime;

use serde::Deserialize;
use serde::Serialize;
use tokio::sync::Notify;

use crate::balancer::status_update::StatusUpdate;
use crate::balancer::upstream_peer::UpstreamPeer;
use crate::errors::result::Result;

#[derive(Serialize, Deserialize)]
pub struct UpstreamPeerPoolInfo {
    pub agents: Vec<UpstreamPeer>,
}

pub struct UpstreamPeerPool {
    pub agents: RwLock<Vec<UpstreamPeer>>,
    pub notifier: Notify,
    pub request_buffer_length: AtomicUsize,
}

impl UpstreamPeerPool {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(Vec::new()),
            notifier: Notify::new(),
            request_buffer_length: AtomicUsize::new(0),
        }
    }

    pub fn info(&self) -> Option<UpstreamPeerPoolInfo> {
        self.agents.read().ok().map(|agents| UpstreamPeerPoolInfo {
            agents: agents.clone(),
        })
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
                upstream_peer.update_status(status_update);
            } else {
                let new_upstream_peer =
                    UpstreamPeer::new_from_status_update(agent_id.to_string(), status_update);

                agents.push(new_upstream_peer);
            }

            agents.sort();

            Ok(())
        })
    }

    pub fn release_slot(&self, agent_id: &str, last_update: SystemTime) -> Result<()> {
        self.with_agents_write(|agents| {
            if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                if peer.last_update < last_update {
                    // edge case, but no need to update anything anyway
                    return Ok(());
                }

                peer.release_slot()?;

                return Ok(());
            }

            Err(format!("There is no agent with id: {agent_id}").into())
        })
    }

    pub fn remove_peer(&self, agent_id: &str) -> Result<()> {
        self.with_agents_write(|agents| {
            if let Some(pos) = agents.iter().position(|p| p.agent_id == agent_id) {
                agents.remove(pos);
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

    #[cfg(feature = "statsd_reporter")]
    /// Returns (slots_idle, slots_processing) tuple.
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

    pub fn use_best_peer(&self) -> Result<Option<UpstreamPeer>> {
        self.with_agents_write(|agents| {
            for peer in agents.iter() {
                if peer.is_usable() {
                    return Ok(Some(peer.clone()));
                }
            }

            Ok(None)
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
    use crate::balancer::test::mock_status_update;

    #[test]
    fn test_race_condition_handling() -> Result<()> {
        let pool = UpstreamPeerPool::new();

        pool.register_status_update("test1", mock_status_update("test1", 5, 0))?;
        pool.with_agents_write(|agents| {
            agents
                .iter_mut()
                .find(|p| p.agent_id == "test1")
                .unwrap()
                .take_slot()
        })?;

        let last_update_at_selection_time = pool.with_agents_read(|agents| {
            Ok(agents
                .iter()
                .find(|p| p.agent_id == "test1")
                .unwrap()
                .last_update)
        })?;

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(peer.slots_idle, 4);
            assert_eq!(peer.slots_processing, 1);
            assert_eq!(peer.slots_taken, 1);

            Ok(())
        })?;

        pool.register_status_update("test1", mock_status_update("test1", 0, 0))?;

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(peer.slots_idle, 0);
            assert_eq!(peer.slots_processing, 0);
            assert_eq!(peer.slots_taken, 1);

            Ok(())
        })?;

        pool.release_slot("test1", last_update_at_selection_time)?;

        pool.with_agents_read(|agents| {
            let peer = agents.iter().find(|p| p.agent_id == "test1").unwrap();
            assert_eq!(peer.slots_idle, 0);
            assert_eq!(peer.slots_processing, 0);
            assert_eq!(peer.slots_taken, 0);

            Ok(())
        })?;

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
        assert_eq!(best_peer.slots_idle, 5);

        Ok(())
    }
}
