use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ConvertsToApplicableState {
    type ApplicableState;

    async fn to_applicable_state(&self) -> Result<Option<Self::ApplicableState>>;
}
