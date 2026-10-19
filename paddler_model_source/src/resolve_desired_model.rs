use std::sync::Arc;

use hf_hub::Cache;
use tokio_util::sync::CancellationToken;

use paddler_agent_status::slot_aggregated_status::SlotAggregatedStatus;
use paddler_cache_dir::cache_dir::CacheDir;
use paddler_messaging::agent_desired_model::AgentDesiredModel;

use crate::desired_model_resolution::DesiredModelResolution;
use crate::huggingface_model_source::HuggingFaceModelSource;
use crate::local_model_path::LocalModelPath;
use crate::model_source::ModelSource;
use crate::model_source_error::ModelSourceError;
use crate::resolves_model_source::ResolvesModelSource;
use crate::url_model_source::UrlModelSource;

pub async fn resolve_desired_model(
    cancellation_token: &CancellationToken,
    cache_dir: &CacheDir,
    desired: &AgentDesiredModel,
    slot_aggregated_status: Arc<SlotAggregatedStatus>,
) -> Result<DesiredModelResolution, ModelSourceError> {
    match desired {
        AgentDesiredModel::None => Ok(DesiredModelResolution::NotConfigured),
        AgentDesiredModel::Uri(uri) => match ModelSource::parse(uri, &slot_aggregated_status)? {
            ModelSource::HuggingFace(reference) => {
                HuggingFaceModelSource {
                    cache: Cache::from_env(),
                    reference,
                }
                .resolve(cancellation_token, slot_aggregated_status)
                .await
            }
            ModelSource::LocalToAgent(agent_local_model_path) => {
                LocalModelPath {
                    path: agent_local_model_path.path().to_path_buf(),
                }
                .resolve(cancellation_token, slot_aggregated_status)
                .await
            }
            ModelSource::Url(download_url) => {
                UrlModelSource {
                    cache_dir: cache_dir.clone(),
                    download_url,
                }
                .resolve(cancellation_token, slot_aggregated_status)
                .await
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::sync::Arc;

    use tempfile::tempdir;
    use tokio_util::sync::CancellationToken;
    use url::Url;

    use paddler_agent_status::slot_aggregated_status::SlotAggregatedStatus;
    use paddler_cache_dir::cache_dir::CacheDir;
    use paddler_messaging::agent_desired_model::AgentDesiredModel;

    use super::resolve_desired_model;
    use crate::desired_model_resolution::DesiredModelResolution;

    #[tokio::test]
    async fn resolves_an_agent_uri_whose_file_name_is_percent_encoded() {
        let model_directory = tempdir().unwrap();
        let model_path = model_directory.path().join("my modèl.gguf");

        File::create(&model_path).unwrap();

        let resolution = resolve_desired_model(
            &CancellationToken::new(),
            &CacheDir::from_process_env(),
            &AgentDesiredModel::Uri(format!(
                "agent://{}/my%20mod%C3%A8l.gguf",
                Url::from_file_path(model_directory.path()).unwrap().path()
            )),
            Arc::new(SlotAggregatedStatus::new(1)),
        )
        .await
        .unwrap();

        assert_eq!(resolution, DesiredModelResolution::Resolved(model_path));
    }
}
