use serde::Deserialize;
use serde::Serialize;

use super::notification_params::RegisterAgentParams;
use super::notification_params::UpdateAgentSlotsParams;

#[derive(Deserialize, Serialize)]
pub enum Notification {
    DeregisterAgent,
    RegisterAgent(RegisterAgentParams),
    UpdateAgentSlots(UpdateAgentSlotsParams),
}
