use serde::Deserialize;
use serde::Serialize;

use super::NotificationParams;
use crate::jsonrpc::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct BadRequestParams {
    pub error: Error,
}

impl NotificationParams for BadRequestParams {}
