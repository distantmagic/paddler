use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::time::SystemTime;

use crate::balancer::status_update::StatusUpdate;
use crate::balancer::upstream_peer::UpstreamPeer;
use crate::errors::result::Result;

#[derive(Serialize, Deserialize)]
pub struct UpstreamPeerPool {
    agents: RwLock<Vec<UpstreamPeer>>,
}

impl UpstreamPeerPool {
    pub fn new() -> Self {
        UpstreamPeerPool {
            agents: RwLock::new(Vec::new()),
        }
    }

    pub fn register_status_update(
        &self,
        agent_id: &str,
        status_update: StatusUpdate,
    ) -> Result<()> {
        match self.agents.write() {
            Ok(mut agents) => {
                if let Some(upstream_peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                    upstream_peer.update_status(status_update);
                } else {
                    let new_upstream_peer =
                        UpstreamPeer::new_from_status_update(agent_id.to_string(), status_update);

                    agents.push(new_upstream_peer);
                }

                agents.sort();

                Ok(())
            }
            Err(_) => Err("Failed to acquire read lock".into()),
        }
    }

    pub fn remove_peer(&self, agent_id: &str) -> Result<()> {
        match self.agents.write() {
            Ok(mut agents) => {
                if let Some(pos) = agents.iter().position(|p| p.agent_id == agent_id) {
                    agents.remove(pos);
                }

                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock".into()),
        }
    }

    pub fn use_best_peer(&self, uses_slots: bool) -> Result<Option<UpstreamPeer>> {
        match self.agents.write() {
            Ok(mut agents) => {
                if let Some(peer) = agents.first_mut() {
                    if peer.slots_idle < 1 {
                        return Ok(None);
                    }

                    if uses_slots {
                        peer.use_slot();
                    }

                    Ok(Some(peer.clone()))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Err("Failed to acquire read lock".into()),
        }
    }

    pub fn release_slot(&self, agent_id: &str) -> Result<bool> {
        let now = SystemTime::now();

        match self.agents.write() {
            Ok(mut agents) => {
                if let Some(peer) = agents.iter_mut().find(|p| p.agent_id == agent_id) {
                    if peer.last_update > now {
                        // edge case, but no need to update anything anyway
                        return Ok(false);
                    }

                    peer.release_slot();

                    return Ok(true);
                }

                Ok(false)
            }
            Err(_) => Err("Failed to acquire write lock".into()),
        }
    }

    pub fn restore_integrity(&self) -> Result<()> {
        match self.agents.write() {
            Ok(mut agents) => {
                agents.sort();

                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock".into()),
        }
    }
}
