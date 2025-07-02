use serde::Deserialize;
use serde::Serialize;

use super::error::Error;
use super::notification_params::BadRequestParams;
use super::notification_params::TooManyRequestsParams;
use super::notification_params::VersionParams;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "notification", content = "content")]
pub enum Notification {
    BadRequest(BadRequestParams),
    TooManyRequests(TooManyRequestsParams),
    Version(VersionParams),
}

impl Notification {
    pub fn bad_request(err: Option<serde_json::Error>) -> Self {
        Self::BadRequest(BadRequestParams {
            error: Error::bad_request(err),
        })
    }

    pub fn too_many_requests() -> Self {
        Self::TooManyRequests(TooManyRequestsParams {})
    }
}
