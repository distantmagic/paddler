use serde::Deserialize;

use super::Notification;
use super::Request;
use crate::jsonrpc::Error;
use crate::jsonrpc::RequestEnvelope;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Message {
    Error(Error),
    Notification(Notification),
    Request(RequestEnvelope<Request>),
}
