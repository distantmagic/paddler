use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AgentStatus {
    pub agent_name: String,
    pub error: Option<String>,
    pub is_decode_error: Option<bool>,
    pub slots_idle: usize,
    pub slots_processing: usize,
}

#[derive(Deserialize, Debug)]
pub struct AgentStatusResponse {
    pub agents: Vec<AgentStatus>,
}
