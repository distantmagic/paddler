use serde::Deserialize;

use super::Notification;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Message {
    Notification(Notification),
}
