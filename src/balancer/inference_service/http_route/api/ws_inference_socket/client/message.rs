use serde::Deserialize;
use serde::Serialize;

use super::Response;
use crate::jsonrpc::Error;
use crate::jsonrpc::ResponseEnvelope;
use crate::rpc_message::RpcMessage;

#[derive(Deserialize, Serialize)]
pub enum Message {
    Error(Error),
    Response(ResponseEnvelope<Response>),
}

impl RpcMessage for Message {}
