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

    pub fn register_status_update(&self, status_update: StatusUpdate) -> Result<()> {
        match self.peers.read() {
            Ok(peers) => {
                println!("Received status update: {:?}", status_update);
                // if let Some(peer) = peers.iter().find(|p| p.agent_id == status_update.agent_id) {
                //     peer.update_status(status_update.status);
                // }

                Ok(())
            }
            Err(_) => Err("Failed to acquire read lock".into()),
        }
    }

    pub fn add_peer(&self, peer: UpstreamPeer) -> Result<()> {
        match self.peers.write() {
            Ok(mut peers) => {
                let pos = peers.partition_point(|inserted| inserted > &peer);

                peers.insert(pos, peer);

                Ok(())
            }
            Err(_) => Err("Failed to acquire write lock".into()),
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
