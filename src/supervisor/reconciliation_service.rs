use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use log::error;
use tokio::sync::broadcast;

use crate::service::Service;
use crate::supervisor::converts_to_applicable_state::ConvertsToApplicableState;
use crate::supervisor::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::supervisor::llamacpp_desired_state::LlamaCppDesiredState;
use crate::supervisor::reconciliation_queue::ReconciliationQueue;

pub struct ReconciliationService {
    llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
    reconciliation_queue: Arc<ReconciliationQueue>,
}

impl ReconciliationService {
    pub fn new(
        llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
        reconciliation_queue: Arc<ReconciliationQueue>,
    ) -> Result<Self> {
        Ok(ReconciliationService {
            llamacpp_applicable_state_holder,
            reconciliation_queue,
        })
    }

    pub async fn on_change_request(
        &self,
        desired_state: Result<LlamaCppDesiredState>,
    ) -> Result<()> {
        let applicable_state = desired_state?.to_applicable_state().await?;

        self.llamacpp_applicable_state_holder
            .set_applicable_state(applicable_state)
    }
}

#[async_trait]
impl Service for ReconciliationService {
    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    debug!("Shutting down monitoring service");

                    return Ok(());
                },
                change_request = self.reconciliation_queue.next_change_request() => {
                    if let Err(err) = self.on_change_request(change_request).await {
                        error!("Failed to apply change request: {err}");
                    }
                }
            }
        }
    }
}
