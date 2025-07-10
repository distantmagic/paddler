use serde::Deserialize;
use serde::Serialize;

use super::notification_params::SetStateParams;
use super::notification_params::VersionParams;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "notification", content = "content")]
pub enum Notification {
    SetState(SetStateParams),
    Version(VersionParams),
}
