use std::time::SystemTime;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SupervisorStatus {
    pub supervisor_name: String,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Supervisor {
    pub last_update: SystemTime,
    pub status: SupervisorStatus,
}

#[derive(Deserialize, Debug)]
pub struct SupervisorsResponse {
    pub supervisors: Vec<Supervisor>,
}
