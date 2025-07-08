use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use log::error;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;

use crate::supervisor::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::supervisor::llamacpp_reconciled_state_holder::LlamaCppReconciledStateHolder;

pub struct LlamaCppProcessService {
    llamacpp_listen_addr: SocketAddr,
    llamacpp_reconciled_state_holder: Arc<LlamaCppReconciledStateHolder>,
    llamacpp_server_bin_path: PathBuf,
}

impl LlamaCppProcessService {
    pub fn new(
        llamacpp_listen_addr: SocketAddr,
        llamacpp_reconciled_state_holder: Arc<LlamaCppReconciledStateHolder>,
        llamacpp_server_bin_path: PathBuf,
    ) -> Result<Self> {
        Ok(LlamaCppProcessService {
            llamacpp_listen_addr,
            llamacpp_reconciled_state_holder,
            llamacpp_server_bin_path,
        })
    }

    async fn on_reconciled_state_change(
        &self,
        llamacpp_applicable_state: Option<LlamaCppApplicableState>,
    ) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl Service for LlamaCppProcessService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
        _listeners_per_fd: usize,
    ) {
        let mut reconciled_state = self.llamacpp_reconciled_state_holder.subscribe();
        let mut ticker = interval(Duration::from_secs(1));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

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
                _ = ticker.tick() => {
                }
            }
        }
    }

    fn name(&self) -> &str {
        "supervisor::llamacpp_process_service"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
