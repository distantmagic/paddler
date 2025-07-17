use serde::Deserialize;
use serde::Serialize;

use super::Notification;
use crate::agent::jsonrpc::Response;
use crate::jsonrpc::Error;
use crate::jsonrpc::ResponseEnvelope;
use crate::rpc_message::RpcMessage;

#[derive(Deserialize, Serialize)]
pub enum Message {
    Error(Error),
    Notification(Notification),
    Response(ResponseEnvelope<Response>),
}

impl RpcMessage for Message {}
