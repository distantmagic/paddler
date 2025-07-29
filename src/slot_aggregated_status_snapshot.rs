use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::agent_issue::AgentIssue;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SlotAggregatedStatusSnapshot {
    pub desired_slots_total: i32,
    pub issues: BTreeSet<AgentIssue>,
    pub model_path: Option<String>,
    pub slots_processing: i32,
    pub slots_total: i32,
    pub version: i32,
}
