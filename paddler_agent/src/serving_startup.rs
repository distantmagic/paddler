use std::future::Future;
use std::future::pending;
use std::mem;
use std::pin::Pin;

use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::finished_serving_startup::FinishedServingStartup;

pub enum ServingStartup {
    Idle,
    Starting {
        cancellation_token: CancellationToken,
        finished: Pin<Box<dyn Future<Output = FinishedServingStartup> + Send>>,
    },
}

impl ServingStartup {
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
