use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::agent_issue::AgentIssue;

#[derive(Deserialize, Serialize)]
pub struct AgentControllerSnapshot {
    pub desired_slots_total: i32,
    pub download_current: usize,
    pub download_filename: Option<String>,
    pub download_total: usize,
    pub id: String,
    pub is_state_applied: bool,
    pub issues: BTreeSet<AgentIssue>,
    pub model_path: Option<String>,
    pub name: Option<String>,
    pub slots_processing: i32,
    pub slots_total: i32,
}
