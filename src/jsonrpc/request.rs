use serde::Deserialize;

use super::request_params::SetStateParams;
use crate::jsonrpc::RequestParams as _;

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
