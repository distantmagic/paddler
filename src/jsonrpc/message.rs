use serde::Deserialize;

use super::notification::Notification;
use super::request::Request;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Notification(Notification),
}
