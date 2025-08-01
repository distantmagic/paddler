use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

use crate::agent_applicable_state::AgentApplicableState;
use crate::agent_desired_model::AgentDesiredModel;
use crate::chat_template::ChatTemplate;
use crate::converts_to_applicable_state::ConvertsToApplicableState;
use crate::inference_parameters::InferenceParameters;
use crate::slot_aggregated_status::SlotAggregatedStatus;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AgentDesiredState {
    pub inference_parameters: InferenceParameters,
    pub model: AgentDesiredModel,
    pub override_chat_template: Option<ChatTemplate>,
}

#[async_trait]
impl ConvertsToApplicableState for AgentDesiredState {
    type ApplicableState = AgentApplicableState;

    async fn to_applicable_state(
        &self,
        slot_aggregated_status: Arc<SlotAggregatedStatus>,
    ) -> Result<Option<Self::ApplicableState>> {
        Ok(Some(AgentApplicableState {
            inference_parameters: self.inference_parameters.clone(),
            model_path: self
                .model
                .to_applicable_state(slot_aggregated_status)
                .await?,
            override_chat_template: None,
        }))
    }
}
