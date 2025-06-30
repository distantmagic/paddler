use serde::Serialize;

use super::super::error::Error;
use super::NotificationParams;

#[derive(Debug, Serialize)]
pub struct BadRequestParams {
    pub error: Error,
}

impl NotificationParams for BadRequestParams {}
