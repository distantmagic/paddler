use serde::Deserialize;

use super::Request;
use crate::jsonrpc::Error;
use crate::jsonrpc::RequestEnvelope;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Message {
    Error(Error),
    Request(RequestEnvelope<Request>),
}
