use std::future::Future;
use std::future::pending;
use std::mem;
use std::pin::Pin;

use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use paddler_agent_runtime::inference_runtime_context::InferenceRuntimeContext;

use crate::finished_serving_startup::FinishedServingStartup;
use crate::serving_startup_attempt::ServingStartupAttempt;

pub enum ServingStartup {
    Idle,
    Starting {
        cancellation_token: CancellationToken,
        finished: Pin<Box<dyn Future<Output = FinishedServingStartup> + Send>>,
    },
}

impl ServingStartup {
    #[must_use]
    pub fn start(
        ServingStartupAttempt {
            agent_applicable_state,
            retry_delay,
            serving_startup_plan,
        }: ServingStartupAttempt,
        agent_shutdown: &CancellationToken,
        inference_runtime_context: InferenceRuntimeContext,
    ) -> Self {
        let cancellation_token = agent_shutdown.child_token();
        let startup_cancellation_token = cancellation_token.clone();

        Self::Starting {
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

    pub fn abandon(&mut self, task_tracker: &TaskTracker) {
        if let Self::Starting {
            cancellation_token,
            finished,
        } = mem::replace(self, Self::Idle)
        {
            cancellation_token.cancel();
            task_tracker.spawn(async move {
                finished.await;
            });
        }
    }

    pub async fn finish(&mut self) -> FinishedServingStartup {
        match self {
            Self::Idle => pending().await,
            Self::Starting { finished, .. } => {
                let finished_serving_startup = finished.as_mut().await;

                *self = Self::Idle;

                finished_serving_startup
            }
        }
    }
}
