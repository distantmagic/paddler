use std::time::SystemTime;

use tokio::process::Child;

#[derive(Debug, Default)]
pub struct StatsdInstance {
    pub child: Option<Child>,
    pub last_update: Option<SystemTime>,
}
