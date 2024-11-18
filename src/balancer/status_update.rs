use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::llamacpp::slot::Slot;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub agent_name: Option<String>,
    pub external_llamacpp_addr: SocketAddr,
    slots: Vec<Slot>,
}

impl StatusUpdate {
    pub fn new(
        agent_name: Option<String>,
        external_llamacpp_addr: SocketAddr,
        slots: Vec<Slot>,
    ) -> Self {
        Self {
            agent_name,
            external_llamacpp_addr,
            slots,
        }
    }

    pub fn get_total_slots_idle(&self) -> usize {
        self.slots.iter().filter(|slot| !slot.is_processing).count()
    }

    pub fn get_total_slots_processing(&self) -> usize {
        self.slots.iter().filter(|slot| slot.is_processing).count()
    }
}

impl actix::Message for StatusUpdate {
    type Result = ();
}
