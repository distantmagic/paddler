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
    pub error: Option<String>,
    pub external_llamacpp_addr: SocketAddr,
    /// None means undetermined, probably due to an error
    pub is_authorized: Option<bool>,
    /// None means undetermined, probably due to an error
    pub is_slots_endpoint_enabled: Option<bool>,
    pub last_update: SystemTime,
    pub quarantined_until: Option<SystemTime>,
    pub slots_idle: usize,
    pub slots_processing: usize,
}

impl UpstreamPeer {
    pub fn new(
        agent_id: String,
        agent_name: Option<String>,
        error: Option<String>,
        external_llamacpp_addr: SocketAddr,
        is_authorized: Option<bool>,
        is_slots_endpoint_enabled: Option<bool>,
        slots_idle: usize,
        slots_processing: usize,
    ) -> Self {
        UpstreamPeer {
            agent_id,
            agent_name,
            error,
            external_llamacpp_addr,
            is_authorized,
            is_slots_endpoint_enabled,
            last_update: SystemTime::now(),
            quarantined_until: None,
            slots_idle,
            slots_processing,
        }
    }

    pub fn new_from_status_update(agent_id: String, status_update: StatusUpdate) -> Self {
        Self::new(
            agent_id,
            status_update.agent_name.to_owned(),
            status_update.error.to_owned(),
            status_update.external_llamacpp_addr,
            status_update.is_authorized,
            status_update.is_slots_endpoint_enabled,
            status_update.idle_slots_count,
            status_update.processing_slots_count,
        )
    }

    pub fn is_usable(&self) -> bool {
        self.slots_idle > 0
            && self.quarantined_until.is_none()
            && self.error.is_none()
            && matches!(self.is_authorized, Some(true))
    }

    pub fn release_slot(&mut self) {
        self.last_update = SystemTime::now();
        self.slots_idle += 1;
        self.slots_processing -= 1;
    }

    pub fn update_status(&mut self, status_update: StatusUpdate) {
        self.agent_name = status_update.agent_name.to_owned();
        self.error = status_update.error.to_owned();
        self.external_llamacpp_addr = status_update.external_llamacpp_addr;
        self.is_authorized = status_update.is_authorized;
        self.is_slots_endpoint_enabled = status_update.is_slots_endpoint_enabled;
        self.last_update = SystemTime::now();
        self.quarantined_until = None;
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
            .is_usable()
            .cmp(&self.is_usable())
            .then_with(|| other.slots_idle.cmp(&self.slots_idle))
            .then_with(|| self.slots_processing.cmp(&other.slots_processing))
            // compare by addr for stable sorting
            .then_with(|| {
                self.external_llamacpp_addr
                    .cmp(&other.external_llamacpp_addr)
            })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    fn create_test_peer() -> UpstreamPeer {
        UpstreamPeer::new(
            "test_agent".to_string(),
            Some("test_name".to_string()),
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            Some(true),
            Some(true),
            5,
            0,
        )
    }

    #[test]
    fn test_take_slot_success() {
        let mut peer = create_test_peer();

        peer.take_slot();

        assert_eq!(peer.slots_idle, 4);
        assert_eq!(peer.slots_processing, 1);
    }

    #[test]
    fn test_take_slot_failure() {
        let mut peer = create_test_peer();

        peer.slots_idle = 0;
        peer.take_slot();

        assert_eq!(peer.slots_idle, 0);
        assert_eq!(peer.slots_processing, 0);
    }

    #[test]
    fn test_release_slot_success() {
        let mut peer = create_test_peer();
        peer.slots_idle = 4;
        peer.slots_processing = 1;

        peer.release_slot();

        assert_eq!(peer.slots_idle, 5);
        assert_eq!(peer.slots_processing, 0);
    }

    #[test]
    fn test_release_slot_failure() {
        let mut peer = create_test_peer();
        peer.slots_processing = 0;

        peer.release_slot();

        assert_eq!(peer.slots_idle, 5);
        assert_eq!(peer.slots_processing, 0);
    }

    #[test]
    fn test_update_status() {
        let mut peer = create_test_peer();
        let status_update = StatusUpdate::new(
            Some("new_name".to_string()),
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            Some(true),
            Some(true),
            vec![],
        );

        peer.update_status(status_update);
        assert_eq!(peer.slots_idle, 0);
        assert_eq!(peer.slots_processing, 0);
        assert_eq!(peer.agent_name, Some("new_name".to_string()));
    }
}
