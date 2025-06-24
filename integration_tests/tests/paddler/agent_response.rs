use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AgentStatusResponse {
    pub agents: Vec<Agent>,
}

#[derive(Deserialize, Debug)]
pub struct Agent {
    pub status: AgentStatus,
}

#[derive(Deserialize, Debug)]
pub struct AgentStatus {
    pub agent_name: String,
    pub error: Option<String>,
    pub is_request_error: Option<bool>,
    pub slots_idle: usize,
    pub slots_processing: usize,
}
