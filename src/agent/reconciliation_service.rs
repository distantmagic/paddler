use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::broadcast;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;

use crate::agent::reconciliation_queue::ReconciliationQueue;
use crate::agent_applicable_state_holder::AgentApplicableStateHolder;
use crate::agent_desired_state::AgentDesiredState;
use crate::converts_to_applicable_state::ConvertsToApplicableState;
use crate::service::Service;

pub struct ReconciliationService {
    agent_applicable_state_holder: Arc<AgentApplicableStateHolder>,
    agent_desired_state: Option<AgentDesiredState>,
    is_converted_to_applicable_state: bool,
    reconciliation_queue: Arc<ReconciliationQueue>,
}

impl ReconciliationService {
    pub fn new(
        agent_applicable_state_holder: Arc<AgentApplicableStateHolder>,
        reconciliation_queue: Arc<ReconciliationQueue>,
    ) -> Result<Self> {
        Ok(ReconciliationService {
            agent_applicable_state_holder,
            agent_desired_state: None,
            is_converted_to_applicable_state: true,
            reconciliation_queue,
        })
    }

    pub async fn convert_to_applicable_state(&mut self) -> Result<()> {
        let applicable_state = match &self.agent_desired_state {
            None => None,
            Some(agent_desired_state) => agent_desired_state.to_applicable_state().await?,
        };

        self.is_converted_to_applicable_state = true;
        self.agent_applicable_state_holder
            .set_applicable_state(applicable_state)
    }

    pub async fn try_convert_to_applicable_state(&mut self) {
        if let Err(err) = self.convert_to_applicable_state().await {
            error!("Failed to convert to applicable state: {err}");
        }
    }
}

#[async_trait]
impl Service for ReconciliationService {
    fn name(&self) -> &'static str {
        "agent::reconciliation_service"
    }

    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut ticker = interval(Duration::from_secs(1));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.recv() => break Ok(()),
                _ = ticker.tick() => {
                    if !self.is_converted_to_applicable_state {
                        self.try_convert_to_applicable_state().await;
                    }
                },
                next_agent_desired_state = self.reconciliation_queue.next_change_request() => {
                    self.is_converted_to_applicable_state = false;
                    self.agent_desired_state = match next_agent_desired_state {
                        Ok(agent_desired_state) => Some(agent_desired_state),
                        Err(err) => {
                            error!("Failed to receive change request from reconciliation queue: {err}");

                            None
                        }
                    };
                    self.try_convert_to_applicable_state().await;
                }
            }
        }
    }
}
