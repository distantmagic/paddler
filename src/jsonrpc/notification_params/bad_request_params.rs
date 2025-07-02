use serde::Deserialize;
use serde::Serialize;

use super::super::error::Error;
use super::NotificationParams;

#[derive(Debug, Deserialize, Serialize)]
pub struct BadRequestParams {
    pub error: Error,
}

impl NotificationParams for BadRequestParams {}
