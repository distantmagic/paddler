use serde::Deserialize;

use super::request_params::DesiredStateParams;
use super::request_params::RequestParams as _;

#[derive(Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {
    SetState(DesiredStateParams),
}

impl Request {
    pub fn request_id(&self) -> String {
        match self {
            Request::SetState(params) => params.request_id(),
        }
    }
}
