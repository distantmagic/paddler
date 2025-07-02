use serde::Deserialize;

use super::request::Request;
use crate::jsonrpc::Notification;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
}
