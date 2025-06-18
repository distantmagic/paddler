use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AgentStatus {
    pub agent_name: String,
    pub slots_idle: usize,
    pub slots_processing: usize,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct AgentStatusResponse {
    pub agents: Vec<AgentStatus>,
}
