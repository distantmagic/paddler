use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

use crate::agent_applicable_state::AgentApplicableState;
use crate::agent_desired_model::AgentDesiredModel;
use crate::converts_to_applicable_state::ConvertsToApplicableState;
use crate::inference_parameters::InferenceParameters;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentDesiredState {
    pub inference_parameters: InferenceParameters,
    pub model: AgentDesiredModel,
}

#[async_trait]
impl ConvertsToApplicableState for AgentDesiredState {
    type ApplicableState = AgentApplicableState;

    async fn to_applicable_state(&self) -> Result<Option<Self::ApplicableState>> {
        let model_path = match self.model.to_applicable_state().await? {
            Some(path) => path,
            None => {
                return Err(anyhow!(
                    "Unable to obtain model path. Make sure that the path is correct."
                ))
            }
        };

        Ok(Some(AgentApplicableState {
            inference_parameters: self.inference_parameters.clone(),
            model_path,
        }))
    }
}
