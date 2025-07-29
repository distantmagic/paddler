use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::slot_aggregated_status::SlotAggregatedStatus;

#[async_trait]
pub trait ConvertsToApplicableState {
    type ApplicableState;

    async fn to_applicable_state(
        &self,
        slot_aggregated_status: Arc<SlotAggregatedStatus>,
    ) -> Result<Option<Self::ApplicableState>>;
}
