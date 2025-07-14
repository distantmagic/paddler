use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct AgentControllerSnapshot {
    pub id: String,
    pub name: Option<String>,
    pub slots_total: usize,
}
