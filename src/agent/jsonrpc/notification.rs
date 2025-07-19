use serde::Deserialize;
use serde::Serialize;

use super::notification_params::SetStateParams;
use super::notification_params::VersionParams;

#[derive(Debug, Deserialize, Serialize)]
pub enum Notification {
    SetState(SetStateParams),
    StopRequest(String),
    Version(VersionParams),
}
