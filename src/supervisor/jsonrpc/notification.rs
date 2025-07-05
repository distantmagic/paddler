use serde::Deserialize;
use serde::Serialize;

use super::notification_params::BadRequestParams;
use super::notification_params::SetStateParams;
use super::notification_params::VersionParams;
use crate::jsonrpc::error::Error;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "notification", content = "content")]
pub enum Notification {
    BadRequest(BadRequestParams),
    SetState(SetStateParams),
    Version(VersionParams),
}

impl Notification {
    pub fn bad_request(err: Option<serde_json::Error>) -> Self {
        Self::BadRequest(BadRequestParams {
            error: Error::bad_request(err),
        })
    }
}
