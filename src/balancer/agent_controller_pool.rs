use anyhow::Result;
use dashmap::DashMap;
use serde::Deserialize;
use serde::Serialize;

use super::agent_controller::AgentController;

#[derive(Deserialize, Serialize)]
pub struct AgentControllerInfo {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct AgentControllerPoolInfo {
    pub agents: Vec<AgentControllerInfo>,
}

pub struct AgentControllerPool {
    agents: DashMap<String, AgentController>,
}

impl AgentControllerPool {
    pub fn new() -> Self {
        AgentControllerPool {
            agents: DashMap::new(),
        }
    }

    pub fn info(&self) -> AgentControllerPoolInfo {
        AgentControllerPoolInfo {
            agents: self
                .agents
                .iter()
                .map(|entry| {
                    let agent = entry.value();

                    AgentControllerInfo {
                        id: agent.id.clone(),
                        name: agent.name.clone(),
                    }
                })
                .collect(),
        }
    }

    pub fn register_agent_controller(
        &self,
        agent_id: String,
        agent: AgentController,
    ) -> Result<()> {
        if self.agents.insert(agent_id, agent).is_none() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("AgentController already registered"))
        }
    }

    pub fn remove_agent_controller(&self, agent_id: &str) -> Result<bool> {
        Ok(self.agents.remove(agent_id).is_some())
    }

    pub fn total_slots(&self) -> Result<(usize, usize)> {
        todo!();
    }

    pub fn total_buffered_requests(&self) -> usize {
        todo!();
    }
}
