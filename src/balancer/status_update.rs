use std::cmp::Eq;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::net::SocketAddr;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub agent_name: Option<String>,
    pub error: Option<String>,
    pub external_llamacpp_addr: SocketAddr,
    pub is_authorized: Option<bool>,
    pub is_connect_error: Option<bool>,
    pub is_decode_error: Option<bool>,
    pub is_deserialize_error: Option<bool>,
    pub is_request_error: Option<bool>,
    pub is_slots_endpoint_enabled: Option<bool>,
    pub is_unexpected_response_status: Option<bool>,
    pub slots_idle: usize,
    pub slots_processing: usize,
    pub model: Option<String>,
}

impl StatusUpdate {
    pub fn has_issues(&self) -> bool {
        self.error.is_some()
            || !self.is_authorized.unwrap_or(false)
            || self.is_connect_error.unwrap_or(true)
            || self.is_decode_error.unwrap_or(true)
            || self.is_deserialize_error.unwrap_or(true)
            || self.is_request_error.unwrap_or(true)
            || self.is_unexpected_response_status.unwrap_or(true)
    }
}

impl actix::Message for StatusUpdate {
    type Result = ();
}

impl Ord for StatusUpdate {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .slots_idle
            .cmp(&self.slots_idle)
            .then_with(|| self.slots_processing.cmp(&other.slots_processing))
            // compare by addr for stable sorting
            .then_with(|| {
                self.external_llamacpp_addr
                    .cmp(&other.external_llamacpp_addr)
            })
    }
}

impl PartialOrd for StatusUpdate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
