use actix_web::web::Data;
use log::error;
use log::info;

use crate::balancer::agent_controller_pool::AgentControllerPool;

pub struct RemoveAgentGuard {
    pub pool: Data<AgentControllerPool>,
    pub agent_id: String,
}

impl Drop for RemoveAgentGuard {
    fn drop(&mut self) {
        if let Err(err) = self.pool.remove_agent_controller(&self.agent_id) {
            error!("Failed to remove agent: {err}");
        }

        info!("Removed agent: {}", self.agent_id);
    }
}
