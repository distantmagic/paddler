use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use hf_hub::api::tokio::Api;
use serde::Deserialize;
use serde::Serialize;

use crate::converts_to_applicable_state::ConvertsToApplicableState;
use crate::huggingface_model_reference::HuggingFaceModelReference;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AgentDesiredModel {
    HuggingFace(HuggingFaceModelReference),
    Local(String),
}

impl AgentDesiredModel {}

#[async_trait]
impl ConvertsToApplicableState for AgentDesiredModel {
    type ApplicableState = PathBuf;

    async fn to_applicable_state(&self) -> Result<Option<Self::ApplicableState>> {
        Ok(match self {
            AgentDesiredModel::HuggingFace(HuggingFaceModelReference {
                filename,
                repo,
            }) => {
                let api = Api::new()?;
                let repo = api.model(repo.to_owned());
                let weights_filename = repo.get(filename).await?;

                Some(weights_filename)
            }
            AgentDesiredModel::Local(path) => Some(PathBuf::from(path)),
        })
    }
}
