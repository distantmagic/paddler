use anyhow::Result;
use async_trait::async_trait;

use crate::llamacpp_desired_state::LlamaCppDesiredState;

#[async_trait]
pub trait SetsDesiredState {
    async fn set_desired_state(&self, desired_state: LlamaCppDesiredState) -> Result<()>;
}
