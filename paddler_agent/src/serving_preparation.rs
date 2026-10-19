use tokio_util::sync::CancellationToken;

use paddler_agent_runtime::inference_runtime_context::InferenceRuntimeContext;

use crate::agent_applicable_state::AgentApplicableState;
use crate::applicable_decisions::ApplicableDecisions;
use crate::capability_starter::CapabilityStarter;
use crate::capability_startup_interruption::CapabilityStartupInterruption;
use crate::model_serving_settings::ModelServingSettings;
use crate::preparation_outcome::PreparationOutcome;
use crate::prepared_serving::PreparedServing;
use crate::retry_delay::RetryDelay;

pub struct ServingPreparation {
    pub agent_applicable_state: AgentApplicableState,
    pub retry_delay: RetryDelay,
}

impl ServingPreparation {
    pub async fn prepare(
        self,
        cancellation_token: CancellationToken,
        inference_runtime_context: InferenceRuntimeContext,
    ) -> PreparationOutcome {
        let Self {
            agent_applicable_state,
            retry_delay,
        } = self;

        retry_delay.elapse(&cancellation_token).await;

        if cancellation_token.is_cancelled() {
            return PreparationOutcome::Cancelled;
        }

        let applicable_decisions = match &agent_applicable_state {
            AgentApplicableState::NotConfigured | AgentApplicableState::WithoutModel { .. } => {
                &ApplicableDecisions::Disabled
            }
            AgentApplicableState::WithModel(ModelServingSettings { capabilities, .. }) => {
                &capabilities.decisions
            }
        };

        match (CapabilityStarter {
            cancellation_token: &cancellation_token,
            inference_runtime_context: &inference_runtime_context,
        })
        .prepare_decisions(applicable_decisions)
        .await
        {
            Ok(prepared_decisions) => PreparationOutcome::Prepared(PreparedServing {
                agent_applicable_state,
                prepared_decisions,
            }),
            Err(CapabilityStartupInterruption::Cancelled) => PreparationOutcome::Cancelled,
            Err(CapabilityStartupInterruption::Failed(error)) => PreparationOutcome::Failed {
                agent_applicable_state,
                error,
            },
        }
    }
}
