use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct AgentControllerSnapshot {
    pub desired_slots_total: i32,
    pub id: String,
    pub model_path: Option<String>,
    pub name: Option<String>,
    pub slots_processing: i32,
    pub slots_total: i32,
}
