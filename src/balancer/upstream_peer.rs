use serde::{Deserialize, Serialize};
use std::{
    cmp::{Eq, Ordering, PartialEq},
    net::SocketAddr,
    time::SystemTime,
};

use crate::balancer::status_update::StatusUpdate;

#[derive(Clone, Debug, Eq, Serialize, Deserialize)]
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

    pub fn new_from_status_update(agent_id: String, status_update: StatusUpdate) -> Self {
        Self::new(
            agent_id,
            status_update.agent_name.to_owned(),
            status_update.external_llamacpp_addr,
            status_update.idle_slots_count,
            status_update.processing_slots_count,
        )
    }

    pub fn release_slot(&mut self) {
        self.last_update = SystemTime::now();
        self.slots_idle += 1;
        self.slots_processing -= 1;
    }

    pub fn update_status(&mut self, status_update: StatusUpdate) {
        self.agent_name = status_update.agent_name.to_owned();
        self.external_llamacpp_addr = status_update.external_llamacpp_addr;
        self.last_update = SystemTime::now();
        self.slots_idle = status_update.idle_slots_count;
        self.slots_processing = status_update.processing_slots_count;
    }

    pub fn take_slot(&mut self) {
        self.last_update = SystemTime::now();
        self.slots_idle -= 1;
        self.slots_processing += 1;
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
