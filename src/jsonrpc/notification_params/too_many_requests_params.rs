use serde::Deserialize;
use serde::Serialize;

use super::NotificationParams;

#[derive(Debug, Deserialize, Serialize)]
pub struct TooManyRequestsParams {}

impl NotificationParams for TooManyRequestsParams {}
