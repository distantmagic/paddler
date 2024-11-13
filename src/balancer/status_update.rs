use serde::{Deserialize, Serialize};

use crate::llamacpp::slot::Slot;

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusUpdate {
    agent_id: uuid::Uuid,
    agent_name: Option<String>,
    slots: Vec<Slot>,
}

impl StatusUpdate {
    pub fn new(agent_id: uuid::Uuid, agent_name: Option<String>, slots: Vec<Slot>) -> Self {
        Self {
            agent_id,
            agent_name,
            slots,
        }
    }
}

impl actix::Message for StatusUpdate {
    type Result = ();
}
