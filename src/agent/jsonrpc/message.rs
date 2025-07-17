use serde::Deserialize;
use serde::Serialize;

use super::Notification;
use super::Request;
use crate::jsonrpc::Error;
use crate::jsonrpc::RequestEnvelope;

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    Error(Error),
    Notification(Notification),
    Request(RequestEnvelope<Request>),
}
