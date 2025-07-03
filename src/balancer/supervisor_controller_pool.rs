use anyhow::Result;
use dashmap::DashMap;
use serde::Deserialize;
use serde::Serialize;

use super::supervisor_controller::SupervisorController;

#[derive(Serialize, Deserialize)]
pub struct SupervisorControllerPoolInfo {
    pub supervisors: Vec<SupervisorController>,
}

pub struct SupervisorControllerPool {
    supervisors: DashMap<String, SupervisorController>,
}

impl SupervisorControllerPool {
    pub fn new() -> Self {
        SupervisorControllerPool {
            supervisors: DashMap::new(),
        }
    }

    pub fn info(&self) -> SupervisorControllerPoolInfo {
        SupervisorControllerPoolInfo {
            supervisors: self
                .supervisors
                .iter()
                .map(|supervisor| supervisor.value().clone())
                .collect(),
        }
    }

    pub fn register_supervisor_controller(
        &self,
        supervisor_id: String,
        supervisor: SupervisorController,
    ) -> Result<()> {
        if self.supervisors.insert(supervisor_id, supervisor).is_none() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("SupervisorController already registered"))
        }
    }

    pub fn remove_supervisor_controller(&self, supervisor_id: &str) -> Result<bool> {
        Ok(self.supervisors.remove(supervisor_id).is_some())
    }
}
