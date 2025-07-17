use serde::Deserialize;

use super::Notification;
use crate::agent::jsonrpc::Response;
use crate::jsonrpc::Error;
use crate::jsonrpc::ResponseEnvelope;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Message {
    Error(Error),
    Notification(Notification),
    Response(ResponseEnvelope<Response>),
}
