use actix_web::web::Data;
use log::error;
use log::info;

use crate::balancer_applicable_state_holder::BalancerApplicableStateHolder;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::chat_template_override_sender_collection::ChatTemplateOverrideSenderCollection;
use crate::balancer::model_metadata_sender_collection::ModelMetadataSenderCollection;

pub struct AgentSocketControllerContext {
    pub balancer_applicable_state_holder: Data<BalancerApplicableStateHolder>,
    pub agent_controller_pool: Data<AgentControllerPool>,
    pub agent_id: String,
    pub chat_template_override_sender_collection: Data<ChatTemplateOverrideSenderCollection>,
    pub generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    pub model_metadata_sender_collection: Data<ModelMetadataSenderCollection>,
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
