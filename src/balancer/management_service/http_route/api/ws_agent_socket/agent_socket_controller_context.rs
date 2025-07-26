use actix_web::web::Data;
use log::error;
use log::info;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::model_metadata_sender_collection::ModelMetadataSenderCollection;
use crate::balancer::state_database::StateDatabase;

pub struct AgentSocketControllerContext {
    pub agent_controller_pool: Data<AgentControllerPool>,
    pub agent_id: String,
    pub generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    pub model_metadata_sender_collection: Data<ModelMetadataSenderCollection>,
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
