use anyhow::Result;
use dashmap::DashMap;
use serde::Deserialize;
use serde::Serialize;

use super::supervisor::Supervisor;

#[derive(Serialize, Deserialize)]
pub struct SupervisorPoolInfo {
    pub supervisors: Vec<Supervisor>,
}

pub struct SupervisorPool {
    supervisors: DashMap<String, Supervisor>,
}

impl SupervisorPool {
    pub fn new() -> Self {
        SupervisorPool {
            supervisors: DashMap::new(),
        }
    }

    pub fn info(&self) -> SupervisorPoolInfo {
        SupervisorPoolInfo {
            supervisors: self
                .supervisors
                .iter()
                .map(|supervisor| supervisor.value().clone())
                .collect(),
        }
    }

    pub fn register_supervisor(&self, supervisor_id: String, supervisor: Supervisor) -> Result<()> {
        if self.supervisors.insert(supervisor_id, supervisor).is_none() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Supervisor already registered"))
        }
    }

    pub fn remove_supervisor(&self, supervisor_id: &str) -> Result<bool> {
        Ok(self.supervisors.remove(supervisor_id).is_some())
    }
}
