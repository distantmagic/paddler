use std::sync::Arc;

use anyhow::Context as _;
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use log::info;
use tokio::sync::broadcast;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;

use crate::agent::llamacpp_arbiter::LlamaCppArbiter;
use crate::agent::llamacpp_arbiter_controller::LlamaCppArbiterController;
use crate::agent::slot_aggregated_status_manager::SlotAggregatedStatusManager;
use crate::agent_applicable_state::AgentApplicableState;
use crate::agent_applicable_state_holder::AgentApplicableStateHolder;
use crate::service::Service;

pub struct LlamaCppArbiterService {
    agent_applicable_state: Option<AgentApplicableState>,
    agent_applicable_state_holder: Arc<AgentApplicableStateHolder>,
    agent_name: Option<String>,
    desired_slots_total: i32,
    is_state_applied: bool,
    llamacpp_arbiter_controller: Option<LlamaCppArbiterController>,
    slot_aggregated_status_manager: Arc<SlotAggregatedStatusManager>,
}

impl LlamaCppArbiterService {
    pub async fn new(
        agent_applicable_state_holder: Arc<AgentApplicableStateHolder>,
        agent_name: Option<String>,
        desired_slots_total: i32,
        slot_aggregated_status_manager: Arc<SlotAggregatedStatusManager>,
    ) -> Result<Self> {
        Ok(LlamaCppArbiterService {
            agent_applicable_state: None,
            agent_applicable_state_holder,
            agent_name,
            desired_slots_total,
            is_state_applied: true,
            llamacpp_arbiter_controller: None,
            slot_aggregated_status_manager,
        })
    }

    async fn apply_state(&mut self) -> Result<()> {
        if let Some(llamacpp_arbiter_controller) = self.llamacpp_arbiter_controller.take() {
            llamacpp_arbiter_controller
                .shutdown()
                .await
                .context("Unable to stop arbiter controller")?;
        }

        if let Some(agent_applicable_state) = self.agent_applicable_state.clone() {
            self.slot_aggregated_status_manager.reset();
            self.llamacpp_arbiter_controller = Some(
                LlamaCppArbiter::new(
                    self.agent_name.clone(),
                    agent_applicable_state,
                    self.desired_slots_total,
                    self.slot_aggregated_status_manager.clone(),
                )
                .spawn()
                .await?,
            );

            info!("Reconciled state change applied successfully");
        }

        self.is_state_applied = true;

        Ok(())
    }

    async fn try_to_apply_state(&mut self) {
        if let Err(err) = self.apply_state().await {
            error!("Failed to apply reconciled state change: {err}");
        }

        self.slot_aggregated_status_manager
            .slot_aggregated_status
            .update_notifier
            .notify_waiters();
    }
}

#[async_trait]
impl Service for LlamaCppArbiterService {
    fn name(&self) -> &'static str {
        "agent::llamacpp_arbiter_service"
    }

    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut reconciled_state = self.agent_applicable_state_holder.subscribe();
        let mut ticker = interval(Duration::from_secs(1));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.recv() => return Ok(()),
                _ = reconciled_state.changed() => {
                    self.agent_applicable_state = reconciled_state.borrow_and_update().clone();
                    self.is_state_applied = false;
                    self.try_to_apply_state().await;
                }
                _ = ticker.tick() => {
                    if !self.is_state_applied {
                        self.try_to_apply_state().await;
                    }
                }
            }
        }
    }
}
