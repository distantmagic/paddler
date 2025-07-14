use serde::Deserialize;
use serde::Serialize;

use super::notification_params::RegisterAgentParams;
use super::notification_params::UpdateAgentStatusParams;

#[derive(Deserialize, Serialize)]
#[serde(tag = "notification", content = "content")]
pub enum Notification {
    DeregisterAgent,
    RegisterAgent(RegisterAgentParams),
    UpdateAgentStatus(UpdateAgentStatusParams),
}
