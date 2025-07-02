use std::time::SystemTime;

use tokio::process::Child;

use crate::metrics::Metrics;

#[derive(Debug, Default)]
pub struct StatsdInstance {
    pub child: Option<Child>,
    pub metrics: Vec<Metrics>,
    pub last_update: Option<SystemTime>,
}
