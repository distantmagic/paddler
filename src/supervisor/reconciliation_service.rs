use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;

use crate::supervisor::reconciliation_queue::ReconciliationQueue;

pub struct ReconciliationService {
    reconciliation_queue: Arc<ReconciliationQueue>,
}

impl ReconciliationService {
    pub fn new(reconciliation_queue: Arc<ReconciliationQueue>) -> Result<Self> {
        Ok(ReconciliationService {
            reconciliation_queue,
        })
    }
}

#[async_trait]
impl Service for ReconciliationService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
        _listeners_per_fd: usize,
    ) {
        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down monitoring service");
                    return;
                },
                change_request = self.reconciliation_queue.next_change_request() => {
                    println!("Reconciliation tick {change_request:?}");
                }
            }
        }
    }

    fn name(&self) -> &str {
        "supervisor::reconciliation"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
