use serde::Deserialize;

use super::request_params::RequestParams as _;
use super::request_params::SetStateParams;

#[derive(Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum Request {
    SetState(SetStateParams),
}

impl Request {
    pub fn request_id(&self) -> String {
        match self {
            Request::SetState(params) => params.request_id(),
        }
    }
}
