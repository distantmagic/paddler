use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::llamacpp::slot::Slot;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
    agent_id: Uuid,
    agent_name: Option<String>,
    external_llamacpp_addr: url::Url,
    slots: Vec<Slot>,
}

impl StatusUpdate {
    pub fn new(
        agent_id: Uuid,
        agent_name: Option<String>,
        external_llamacpp_addr: url::Url,
        slots: Vec<Slot>,
    ) -> Self {
        Self {
            agent_id,
            agent_name,
            external_llamacpp_addr,
            slots,
        }
    }
}

impl actix::Message for StatusUpdate {
    type Result = ();
}
