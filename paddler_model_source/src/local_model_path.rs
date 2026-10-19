use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::fs::try_exists;
use tokio_util::sync::CancellationToken;

use paddler_agent_status::slot_aggregated_status::SlotAggregatedStatus;

use crate::desired_model_resolution::DesiredModelResolution;
use crate::model_source_error::ModelSourceError;
use crate::resolves_model_source::ResolvesModelSource;

pub struct LocalModelPath {
    pub path: PathBuf,
}

#[async_trait]
impl ResolvesModelSource for LocalModelPath {
    async fn resolve(
        &self,
        _cancellation_token: &CancellationToken,
        _slot_aggregated_status: Arc<SlotAggregatedStatus>,
    ) -> Result<DesiredModelResolution, ModelSourceError> {
        match try_exists(&self.path).await {
            Ok(true) => Ok(DesiredModelResolution::Resolved(self.path.clone())),
            Ok(false) => Ok(DesiredModelResolution::LocalFileMissing(self.path.clone())),
            Err(source) => Err(ModelSourceError::LocalFileUncheckable {
                path: self.path.clone(),
                source,
            }),
        }
    }
}
