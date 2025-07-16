use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::Notify;

use super::agent_controller::AgentController;
use super::agent_controller_pool_total_slots::AgentControllerPoolTotalSlots;
use crate::balancer::agent_controller_pool_snapshot::AgentControllerPoolSnapshot;
use crate::balancer::agent_controller_snapshot::AgentControllerSnapshot;
use crate::produces_snapshot::ProducesSnapshot;

pub struct AgentControllerPool {
    agents: DashMap<String, Arc<AgentController>>,
    pub update_notifier: Notify,
}

impl AgentControllerPool {
    pub fn new() -> Self {
        AgentControllerPool {
            agents: DashMap::new(),
            update_notifier: Notify::new(),
        }
    }

    pub fn get_agent_controller(&self, agent_id: &str) -> Option<Arc<AgentController>> {
        self.agents.get(agent_id).map(|entry| entry.value().clone())
    }

    pub fn register_agent_controller(
        &self,
        agent_id: String,
        agent: Arc<AgentController>,
    ) -> Result<()> {
        if self.agents.insert(agent_id, agent).is_none() {
            self.update_notifier.notify_waiters();

            Ok(())
        } else {
            Err(anyhow::anyhow!("AgentController already registered"))
        }
    }

    pub fn remove_agent_controller(&self, agent_id: &str) -> Result<bool> {
        if self.agents.remove(agent_id).is_some() {
            self.update_notifier.notify_waiters();

            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn total_slots(&self) -> AgentControllerPoolTotalSlots {
        let mut slots_processing = 0;
        let mut slots_total = 0;

        for entry in self.agents.iter() {
            let agent = entry.value();

            slots_processing += agent.slots_processing.get();
            slots_total += agent.slots_total;
        }

        AgentControllerPoolTotalSlots {
            slots_processing,
            slots_total,
        }
    }

    pub fn total_buffered_requests(&self) -> usize {
        todo!();
    }
}

impl ProducesSnapshot for AgentControllerPool {
    type Snapshot = AgentControllerPoolSnapshot;

    fn make_snapshot(&self) -> Self::Snapshot {
        let agents: Vec<AgentControllerSnapshot> = self
            .agents
            .iter()
            .map(|entry| entry.value().make_snapshot())
            .collect();

        AgentControllerPoolSnapshot {
            agents,
        }
    }
}
