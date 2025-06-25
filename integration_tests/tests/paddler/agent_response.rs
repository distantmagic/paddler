use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct AgentStatus {
    pub agent_name: String,
    pub slots_idle: usize,
    pub slots_processing: usize,
    pub is_connect_error: Option<bool>,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Agent {
    pub status: AgentStatus,
}

#[derive(Deserialize, Debug)]
pub struct AgentsResponse {
    pub agents: Vec<Agent>,
}
