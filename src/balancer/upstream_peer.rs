use serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ordering, PartialEq};
use std::net::SocketAddr;
use std::time::SystemTime;

#[derive(Clone, Eq, Serialize, Deserialize)]
pub struct UpstreamPeer {
    pub agent_id: String,
    pub agent_name: Option<String>,
    pub external_llamacpp_addr: SocketAddr,
    pub last_update: SystemTime,
    pub slots_idle: usize,
    pub slots_processing: usize,
}

impl UpstreamPeer {
    pub fn new(
        agent_id: String,
        agent_name: Option<String>,
        external_llamacpp_addr: SocketAddr,
        slots_idle: usize,
        slots_processing: usize,
    ) -> Self {
        UpstreamPeer {
            agent_id,
            agent_name,
            external_llamacpp_addr,
            last_update: SystemTime::now(),
            slots_idle,
            slots_processing,
        }
    }
}

impl Ord for UpstreamPeer {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .slots_idle
            .cmp(&self.slots_idle)
            .then_with(|| self.slots_processing.cmp(&other.slots_processing))
            // compare by id for stable sorting
            .then_with(|| self.agent_id.cmp(&other.agent_id))
    }
}

impl PartialEq for UpstreamPeer {
    fn eq(&self, other: &Self) -> bool {
        self.agent_id == other.agent_id
    }
}

impl PartialOrd for UpstreamPeer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
