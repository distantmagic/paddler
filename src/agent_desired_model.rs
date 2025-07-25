use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use hf_hub::api::tokio::Api;
use hf_hub::api::tokio::ApiError;
use hf_hub::Repo;
use hf_hub::RepoType;
use log::warn;
use serde::Deserialize;
use serde::Serialize;
use tokio::time::sleep;
use tokio::time::Duration;

use crate::converts_to_applicable_state::ConvertsToApplicableState;
use crate::huggingface_model_reference::HuggingFaceModelReference;

const LOCK_RETRY_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AgentDesiredModel {
    HuggingFace(HuggingFaceModelReference),
    Local(String),
}

#[async_trait]
impl ConvertsToApplicableState for AgentDesiredModel {
    type ApplicableState = PathBuf;

    async fn to_applicable_state(&self) -> Result<Option<Self::ApplicableState>> {
        Ok(match self {
            AgentDesiredModel::HuggingFace(HuggingFaceModelReference {
                filename,
                repo_id,
                revision,
            }) => {
                let hf_api = Api::new()?;
                let hf_repo = hf_api.repo(Repo::with_revision(
                    repo_id.to_owned(),
                    RepoType::Model,
                    revision.to_owned(),
                ));

                let weights_filename = match hf_repo.get(filename).await {
                    Ok(resolved_filename) => resolved_filename,
                    Err(ApiError::LockAcquisition(lock_path)) => {
                        warn!(
                            "Waiting to acquire download lock for '{}'. Sleeping for {} secs",
                            lock_path.display(),
                            LOCK_RETRY_TIMEOUT.as_secs()
                        );

                        sleep(LOCK_RETRY_TIMEOUT).await;

                        return Err(anyhow!("Failed to acquire download lock '{}'. Is more than one agent running on this machine?", lock_path.display()));
                    }
                    Err(err_other) => return Err(err_other.into()),
                };

                Some(weights_filename)
            }
            AgentDesiredModel::Local(path) => Some(PathBuf::from(path)),
        })
    }
}
