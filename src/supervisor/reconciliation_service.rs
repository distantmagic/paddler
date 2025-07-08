use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use log::error;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;

use crate::supervisor::llamacpp_desired_state::LlamaCppDesiredState;
use crate::supervisor::reconciliation_queue::ReconciliationQueue;

pub struct ReconciliationService {
    llamacpp_listen_addr: SocketAddr,
    reconciliation_queue: Arc<ReconciliationQueue>,
}

impl ReconciliationService {
    pub fn new(
        llamacpp_listen_addr: SocketAddr,
        reconciliation_queue: Arc<ReconciliationQueue>,
    ) -> Result<Self> {
        Ok(ReconciliationService {
            llamacpp_listen_addr,
            reconciliation_queue,
        })
    }

    pub async fn on_change_request(
        &self,
        desired_state: Result<LlamaCppDesiredState>,
    ) -> Result<()> {
        Ok(())
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
                    if let Err(err) = self.on_change_request(change_request).await {
                        error!("Failed to apply change request: {err}");
                    }
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
