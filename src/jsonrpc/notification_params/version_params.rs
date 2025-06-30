use serde::Serialize;

use super::NotificationParams;

#[derive(Debug, Serialize)]
pub struct VersionParams {
    pub version: String,
}

impl NotificationParams for VersionParams {}
