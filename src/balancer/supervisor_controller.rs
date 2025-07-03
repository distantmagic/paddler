use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Deserialize, Serialize)]
pub struct SupervisorController {
    pub id: String,
    pub name: Option<String>,
}

impl SupervisorController {
    pub fn new(id: String, name: Option<String>) -> Self {
        SupervisorController {
            id,
            name,
        }
    }
}
