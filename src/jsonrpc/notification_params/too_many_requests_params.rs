use serde::Serialize;

use super::NotificationParams;

#[derive(Debug, Serialize)]
pub struct TooManyRequestsParams {}

impl NotificationParams for TooManyRequestsParams {}
