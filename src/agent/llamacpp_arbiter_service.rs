use std::sync::Arc;

use anyhow::Context as _;
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use log::info;
use tokio::sync::broadcast;

use crate::agent::llamacpp_arbiter::LlamaCppArbiter;
use crate::agent::llamacpp_arbiter_controller::LlamaCppArbiterController;
use crate::agent::slot_aggregated_status_manager::SlotAggregatedStatusManager;
use crate::agent_applicable_state::AgentApplicableState;
use crate::agent_applicable_state_holder::AgentApplicableStateHolder;
use crate::service::Service;

pub struct LlamaCppArbiterService {
    agent_applicable_state_holder: Arc<AgentApplicableStateHolder>,
    llamacpp_arbiter_controller: Option<LlamaCppArbiterController>,
    slot_aggregated_status_manager: Arc<SlotAggregatedStatusManager>,
    slots_total: i32,
}

impl LlamaCppArbiterService {
    pub async fn new(
        agent_applicable_state_holder: Arc<AgentApplicableStateHolder>,
        slot_aggregated_status_manager: Arc<SlotAggregatedStatusManager>,
        slots_total: i32,
    ) -> Result<Self> {
        Ok(LlamaCppArbiterService {
            agent_applicable_state_holder,
            llamacpp_arbiter_controller: None,
            slot_aggregated_status_manager,
            slots_total,
        })
    }

    async fn on_reconciled_state_change(
        &mut self,
        agent_applicable_state: Option<AgentApplicableState>,
    ) -> Result<()> {
        if let Some(llamacpp_arbiter_controller) = self.llamacpp_arbiter_controller.take() {
            llamacpp_arbiter_controller
                .shutdown()
                .await
                .context("Unable to stop arbiter controller")?;
        }

        if let Some(agent_applicable_state) = agent_applicable_state {
            self.slot_aggregated_status_manager.reset();
            self.llamacpp_arbiter_controller = Some(
                LlamaCppArbiter::new(
                    agent_applicable_state,
                    self.slot_aggregated_status_manager.clone(),
                    self.slots_total,
                )
                .spawn()
                .await
                .context("Unable to spawn arbiter")?,
            );

            info!("Reconciled state change applied successfully");
        }

        Ok(())
    }
}

#[async_trait]
impl Service for LlamaCppArbiterService {
    fn name(&self) -> &'static str {
        "agent::llamacpp_arbiter_service"
    }

    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut reconciled_state = self.agent_applicable_state_holder.subscribe();

        loop {
            tokio::select! {
                _ = shutdown.recv() => return Ok(()),
                _ = reconciled_state.changed() => {
                    let agent_applicable_state: Option<AgentApplicableState> = reconciled_state.borrow_and_update().clone();

                    if let Err(err) = self.on_reconciled_state_change(agent_applicable_state).await {
                        error!("Failed to apply reconciled state change: {err}");
                    }
                }
            }
        }
    }
}
