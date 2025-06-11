use std::net::SocketAddr;

use serde::Deserialize;
use serde::Serialize;

use crate::llamacpp::slot::Slot;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub agent_name: Option<String>,
    pub error: Option<String>,
    pub external_llamacpp_addr: SocketAddr,
    pub idle_slots_count: usize,
    pub is_authorized: Option<bool>,
    pub is_slots_endpoint_enabled: Option<bool>,
    pub processing_slots_count: usize,
    slots: Vec<Slot>,
}

impl StatusUpdate {
    pub fn new(
        agent_name: Option<String>,
        error: Option<String>,
        external_llamacpp_addr: SocketAddr,
        is_authorized: Option<bool>,
        is_slots_endpoint_enabled: Option<bool>,
        slots: Vec<Slot>,
    ) -> Self {
        let idle_slots_count = slots.iter().filter(|slot| !slot.is_processing).count();

        Self {
            agent_name,
            error,
            external_llamacpp_addr,
            idle_slots_count,
            is_authorized,
            is_slots_endpoint_enabled,
            processing_slots_count: slots.len() - idle_slots_count,
            slots,
        }
    }
}

impl actix::Message for StatusUpdate {
    type Result = ();
}
