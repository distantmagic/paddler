use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

use crate::agent_applicable_state::AgentApplicableState;
use crate::agent_desired_model::AgentDesiredModel;
use crate::converts_to_applicable_state::ConvertsToApplicableState;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentDesiredState {
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

        Ok(Some(AgentApplicableState { model_path }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_desired_model::AgentDesiredModel;
    use crate::huggingface_model_reference::HuggingFaceModelReference;

    #[test]
    fn test_serialization() -> Result<()> {
        let desired_state = AgentDesiredState {
            model: AgentDesiredModel::HuggingFace(HuggingFaceModelReference {
                filename: "model.gguf".to_string(),
                repo_id: "org/repo".to_string(),
                revision: "main".to_string(),
            }),
        };

        let serialized = serde_json::to_string(&desired_state)?;

        assert_eq!(
            serialized,
            r#"{"model":{"HuggingFace":{"filename":"model.gguf","repo_id":"org/repo","revision":"main"}}}"#
        );

        Ok(())
    }
}
