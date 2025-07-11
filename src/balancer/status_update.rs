use std::cmp::Eq;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::net::SocketAddr;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub slots_idle: usize,
    pub slots_processing: usize,
}

impl Ord for StatusUpdate {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .slots_idle
            .cmp(&self.slots_idle)
            .then_with(|| self.slots_processing.cmp(&other.slots_processing))
    }
}

impl PartialOrd for StatusUpdate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
