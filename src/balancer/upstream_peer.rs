use std::cmp::Eq;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::net::SocketAddr;
use std::time::SystemTime;

use serde::Deserialize;
use serde::Serialize;

use crate::balancer::status_update::StatusUpdate;
use crate::errors::result::Result;

#[derive(Clone, Debug, Eq, Serialize, Deserialize)]
pub struct UpstreamPeer {
    pub agent_id: String,
    pub agent_name: Option<String>,
    pub error: Option<String>,
    pub external_llamacpp_addr: SocketAddr,
    /// None means undetermined, probably due to an error
    pub is_authorized: Option<bool>,
    pub is_unexpected_response_status: Option<bool>,
    pub is_connect_error: Option<bool>,
    pub is_decode_error: Option<bool>,
    pub is_deserialize_error: Option<bool>,
    pub is_request_error: Option<bool>,
    /// None means undetermined, probably due to an error
    pub is_slots_endpoint_enabled: Option<bool>,
    pub last_update: SystemTime,
    pub quarantined_until: Option<SystemTime>,
    pub slots_idle: usize,
    pub slots_processing: usize,
    pub slots_taken: usize,
    pub slots_taken_since_last_status_update: usize,
}

impl UpstreamPeer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        agent_id: String,
        agent_name: Option<String>,
        error: Option<String>,
        is_unexpected_response_status: Option<bool>,
        is_connect_error: Option<bool>,
        is_decode_error: Option<bool>,
        is_deserialize_error: Option<bool>,
        is_request_error: Option<bool>,
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
            is_unexpected_response_status,
            is_connect_error,
            is_decode_error,
            is_deserialize_error,
            is_request_error,
            external_llamacpp_addr,
            is_authorized,
            is_slots_endpoint_enabled,
            last_update: SystemTime::now(),
            quarantined_until: None,
            slots_idle,
            slots_processing,
            slots_taken: 0,
            slots_taken_since_last_status_update: 0,
        }
    }

    pub fn new_from_status_update(agent_id: String, status_update: StatusUpdate) -> Self {
        Self::new(
            agent_id,
            status_update.agent_name.to_owned(),
            status_update.error.to_owned(),
            status_update.is_unexpected_response_status,
            status_update.is_connect_error,
            status_update.is_decode_error,
            status_update.is_deserialize_error,
            status_update.is_request_error,
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

    pub fn release_slot(&mut self) -> Result<()> {
        if self.slots_taken < 1 {
            return Err("Cannot release a slot when there are no taken slots".into());
        }

        self.last_update = SystemTime::now();
        self.slots_taken -= 1;

        if self.slots_taken_since_last_status_update > 0 {
            self.slots_taken_since_last_status_update -= 1;
            self.slots_idle += 1;
            self.slots_processing -= 1;
        }

        Ok(())
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
        self.slots_taken_since_last_status_update = 0;
    }

    pub fn take_slot(&mut self) -> Result<()> {
        if self.slots_idle < 1 {
            return Err("Cannot take a slot when there are no idle slots".into());
        }

        self.last_update = SystemTime::now();
        self.slots_taken_since_last_status_update += 1;
        self.slots_taken += 1;
        self.slots_idle -= 1;
        self.slots_processing += 1;

        Ok(())
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
    use std::net::IpAddr;
    use std::net::Ipv4Addr;
    use std::net::SocketAddr;

    use super::*;

    fn create_test_peer() -> UpstreamPeer {
        UpstreamPeer::new(
            "test_agent".to_string(),
            Some("test_name".to_string()),
            None,
            None,
            None,
            None,
            None,
            None,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            Some(true),
            Some(true),
            5,
            0,
        )
    }

    #[test]
    fn test_take_release_slot() -> Result<()> {
        let mut peer = create_test_peer();

        assert_eq!(peer.slots_idle, 5);
        assert_eq!(peer.slots_processing, 0);

        peer.take_slot()?;

        assert_eq!(peer.slots_idle, 4);
        assert_eq!(peer.slots_processing, 1);

        peer.release_slot()?;

        assert_eq!(peer.slots_idle, 5);
        assert_eq!(peer.slots_processing, 0);

        Ok(())
    }

    #[test]
    fn test_take_slot_failure() {
        let mut peer = create_test_peer();

        peer.slots_idle = 0;

        assert!(peer.take_slot().is_err());
    }

    #[test]
    fn test_release_slot_failure() -> Result<()> {
        let mut peer = create_test_peer();

        peer.slots_processing = 0;

        assert!(peer.release_slot().is_err());

        Ok(())
    }

    #[test]
    fn test_update_status() {
        let mut peer = create_test_peer();
        let status_update = StatusUpdate::new(
            Some("new_name".to_string()),
            None,
            None,
            None,
            None,
            None,
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
