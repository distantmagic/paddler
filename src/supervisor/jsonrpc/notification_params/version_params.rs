use serde::Deserialize;
use serde::Serialize;

use super::NotificationParams;

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionParams {
    pub version: String,
}

impl NotificationParams for VersionParams {}
