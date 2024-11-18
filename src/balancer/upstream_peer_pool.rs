use std::sync::RwLock;

use crate::balancer::status_update::StatusUpdate;
use crate::balancer::upstream_peer::UpstreamPeer;
use crate::errors::result::Result;

pub struct UpstreamPeerPool {
    peers: RwLock<Vec<UpstreamPeer>>,
}

impl UpstreamPeerPool {
    pub fn new() -> Self {
        UpstreamPeerPool {
            peers: RwLock::new(Vec::new()),
        }
    }

    pub fn get_cloned_peers(&self) -> Result<Vec<UpstreamPeer>> {
        match self.peers.read() {
            Ok(peers) => Ok(peers.clone()),
            Err(_) => Err("Failed to acquire read lock".into()),
        }
    }

    pub fn register_status_update(
        &self,
        agent_id: &str,
        status_update: StatusUpdate,
    ) -> Result<()> {
        match self.peers.write() {
            Ok(mut peers) => {
                if let Some(upstream_peer) = peers.iter_mut().find(|p| p.agent_id == agent_id) {
                    upstream_peer.agent_name = status_update.agent_name.clone();
                    upstream_peer.external_llamacpp_addr = status_update.external_llamacpp_addr;
                    upstream_peer.slots_idle = status_update.get_total_slots_idle();
                    upstream_peer.slots_processing = status_update.get_total_slots_processing();

                    peers.sort();
                } else {
                    let new_upstream_peer = UpstreamPeer::new(
                        agent_id.to_string(),
                        status_update.agent_name.clone(),
                        status_update.external_llamacpp_addr,
                        status_update.get_total_slots_idle(),
                        status_update.get_total_slots_processing(),
                    );

                    let pos = peers.partition_point(|inserted| inserted > &new_upstream_peer);

                    peers.insert(pos, new_upstream_peer);
                }

                Ok(())
            }
            Err(_) => Err("Failed to acquire read lock".into()),
        }
    }

    pub fn remove_peer(&self, agent_id: &str) -> Result<()> {
        match self.peers.write() {
            Ok(mut peers) => {
                if let Some(pos) = peers.iter().position(|p| p.agent_id == agent_id) {
                    peers.remove(pos);
                }

                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock".into()),
        }
    }
}
