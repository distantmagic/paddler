use serde::Deserialize;

use super::Notification;
use crate::jsonrpc::Error;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Message {
    Error(Error),
    Notification(Notification),
}
