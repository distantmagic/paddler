use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RegisterAgentParams {
    pub name: Option<String>,
    pub slots_total: i32,
}
