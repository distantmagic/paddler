use serde::Deserialize;
use serde::Serialize;

use super::notification_params::RegisterSupervisorParams;

#[derive(Deserialize, Serialize)]
#[serde(tag = "notification", content = "content")]
pub enum Notification {
    RegisterSupervisor(RegisterSupervisorParams),
}
