use tokio_util::sync::CancellationToken;

use paddler_agent_runtime::inference_runtime_context::InferenceRuntimeContext;

use crate::agent_applicable_state::AgentApplicableState;
use crate::finished_serving_startup::FinishedServingStartup;
use crate::retry_delay::RetryDelay;
use crate::serving_startup::ServingStartup;
use crate::serving_startup_plan::ServingStartupPlan;

pub struct ServingStartupAttempt {
    pub agent_applicable_state: AgentApplicableState,
    pub retry_delay: RetryDelay,
    pub serving_startup_plan: ServingStartupPlan,
}

impl ServingStartupAttempt {
    #[must_use]
    pub fn start(
        self,
        agent_shutdown: &CancellationToken,
        inference_runtime_context: InferenceRuntimeContext,
    ) -> ServingStartup {
        let Self {
            agent_applicable_state,
            retry_delay,
            serving_startup_plan,
        } = self;
        let cancellation_token = agent_shutdown.child_token();
        let startup_cancellation_token = cancellation_token.clone();

        ServingStartup::Starting {
            cancellation_token,
            finished: Box::pin(async move {
                retry_delay.elapse(&startup_cancellation_token).await;

                FinishedServingStartup {
                    outcome: serving_startup_plan
                        .start(&startup_cancellation_token, &inference_runtime_context)
                        .await,
                    agent_applicable_state,
                }
            }),
        }
    }
}
