use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use log::error;
use tokio::sync::broadcast;

use crate::agent::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::agent::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::agent::llamacpp_arbiter::LlamaCppArbiter;
use crate::service::Service;

pub struct LlamaCppArbiterService {
    llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
    llamacpp_arbiter: Option<LlamaCppArbiter>,
}

impl LlamaCppArbiterService {
    pub fn new(
        llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
    ) -> Result<Self> {
        Ok(LlamaCppArbiterService {
            llamacpp_applicable_state_holder,
            llamacpp_arbiter: None,
        })
    }

    async fn on_reconciled_state_change(
        &mut self,
        llamacpp_applicable_state: Option<LlamaCppApplicableState>,
    ) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl Service for LlamaCppArbiterService {
    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut reconciled_state = self.llamacpp_applicable_state_holder.subscribe();

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    debug!("Shutting down monitoring service");

                    return Ok(());
                },
                _ = reconciled_state.changed() => {
                    let llamacpp_applicable_state: Option<LlamaCppApplicableState> = reconciled_state.borrow_and_update().clone();

                    if let Err(err) = self.on_reconciled_state_change(llamacpp_applicable_state).await {
                        error!("Failed to apply reconciled state change: {err}");
                    }
                }
            }
        }
    }
}
