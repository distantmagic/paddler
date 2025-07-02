use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize)]
pub struct Supervisor {
    pub id: String,
    pub name: Option<String>,
}
