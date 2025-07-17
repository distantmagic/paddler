use serde::Deserialize;
use serde::Serialize;

use super::Notification;
use crate::agent::jsonrpc::Response;
use crate::jsonrpc::Error;
use crate::jsonrpc::ResponseEnvelope;

#[derive(Deserialize, Serialize)]
pub enum Message {
    Error(Error),
    Notification(Notification),
    Response(ResponseEnvelope<Response>),
}
