use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

use crate::converts_to_applicable_state::ConvertsToApplicableState;
use crate::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::llamacpp_desired_model::LlamaCppDesiredModel;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct LlamaCppDesiredState {
    pub model: LlamaCppDesiredModel,
}

#[async_trait]
impl ConvertsToApplicableState for LlamaCppDesiredState {
    type ApplicableState = LlamaCppApplicableState;

    async fn to_applicable_state(&self) -> Result<Option<Self::ApplicableState>> {
        let model_path = match self.model.to_applicable_state().await? {
            Some(path) => path,
            None => {
                return Err(anyhow!(
                    "Unable to obtain model path. Make sure that the path is correct."
                ))
            }
        };

        Ok(Some(LlamaCppApplicableState {
            model_path,
        }))
    }
}
