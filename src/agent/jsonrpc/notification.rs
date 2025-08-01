use serde::Deserialize;
use serde::Serialize;

use super::notification_params::SetStateParams;
use super::notification_params::VersionParams;

#[derive(Debug, Deserialize, Serialize)]
pub enum Notification {
    StopGeneratingTokens(String),
    SetState(SetStateParams),
    Version(VersionParams),
}
