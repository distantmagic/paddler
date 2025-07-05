use serde::Deserialize;
use serde::Serialize;

use crate::jsonrpc::NotificationParams;

#[derive(Debug, Deserialize, Serialize)]
pub struct VersionParams {
    pub version: String,
}

impl NotificationParams for VersionParams {}
