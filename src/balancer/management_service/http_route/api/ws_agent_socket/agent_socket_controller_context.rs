use actix_web::web::Data;
use log::error;
use log::info;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::state_database::StateDatabase;

pub struct AgentSocketControllerContext {
    pub agent_controller_pool: Data<AgentControllerPool>,
    pub agent_id: String,
    pub state_database: Data<dyn StateDatabase>,
}

impl Drop for AgentSocketControllerContext {
    fn drop(&mut self) {
        if let Err(err) = self
            .agent_controller_pool
            .remove_agent_controller(&self.agent_id)
        {
            error!("Failed to remove agent: {err}");
        }

        info!("Removed agent: {}", self.agent_id);
    }
}
