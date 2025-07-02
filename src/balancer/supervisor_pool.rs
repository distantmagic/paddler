use anyhow::Result;
use dashmap::DashSet;

pub struct SupervisorPool {
    supervisors: DashSet<String>,
}

impl SupervisorPool {
    pub fn new() -> Self {
        SupervisorPool {
            supervisors: DashSet::new(),
        }
    }

    pub fn register_supervisor(&self, supervisor_id: String) -> Result<()> {
        if self.supervisors.insert(supervisor_id) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Supervisor already registered"))
        }
    }

    pub fn remove_supervisor(&self, supervisor_id: &str) -> Result<bool> {
        Ok(self.supervisors.remove(supervisor_id).is_some())
    }
}
