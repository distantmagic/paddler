use serde::Deserialize;
use serde::Serialize;

use crate::jsonrpc::error::Error;
use crate::jsonrpc::NotificationParams;

#[derive(Debug, Deserialize, Serialize)]
pub struct BadRequestParams {
    pub error: Error,
}

impl NotificationParams for BadRequestParams {}
