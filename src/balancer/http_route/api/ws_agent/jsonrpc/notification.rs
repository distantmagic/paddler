use serde::Deserialize;
use serde::Serialize;

use super::notification_params::RegisterAgentParams;

#[derive(Deserialize, Serialize)]
#[serde(tag = "notification", content = "content")]
pub enum Notification {
    RegisterAgent(RegisterAgentParams),
}
