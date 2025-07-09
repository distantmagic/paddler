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

use crate::supervisor::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::supervisor::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::supervisor::llamacpp_arbiter::LlamaCppArbiter;

pub struct LlamaCppArbiterService {
    llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
    llamacpp_arbiter: Option<LlamaCppArbiter>,
    llamacpp_listen_addr: SocketAddr,
}

impl LlamaCppArbiterService {
    pub fn new(
        llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
        llamacpp_listen_addr: SocketAddr,
    ) -> Result<Self> {
        Ok(LlamaCppArbiterService {
            llamacpp_applicable_state_holder,
            llamacpp_arbiter: None,
            llamacpp_listen_addr,
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
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
        _listeners_per_fd: usize,
    ) {
        let mut reconciled_state = self.llamacpp_applicable_state_holder.subscribe();

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down monitoring service");
                    return;
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

    fn name(&self) -> &str {
        "supervisor::llamacpp_arbiter_service"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
