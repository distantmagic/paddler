use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use log::error;
use nix::sys::signal::Signal;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;

use crate::supervisor::llamacpp_applicable_state::LlamaCppApplicableState;
use crate::supervisor::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::supervisor::llamacpp_process::LlamaCppProcess;

pub struct LlamaCppProcessService {
    llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
    llamacpp_listen_addr: SocketAddr,
    llamacpp_process: Option<LlamaCppProcess>,
    llamacpp_server_bin_path: PathBuf,
}

impl LlamaCppProcessService {
    pub fn new(
        llamacpp_applicable_state_holder: Arc<LlamaCppApplicableStateHolder>,
        llamacpp_listen_addr: SocketAddr,
        llamacpp_server_bin_path: PathBuf,
    ) -> Result<Self> {
        Ok(LlamaCppProcessService {
            llamacpp_applicable_state_holder,
            llamacpp_listen_addr,
            llamacpp_process: None,
            llamacpp_server_bin_path,
        })
    }

    async fn on_reconciled_state_change(
        &mut self,
        llamacpp_applicable_state: Option<LlamaCppApplicableState>,
    ) -> Result<()> {
        if let Some(llamacpp_process) = &self.llamacpp_process {
            llamacpp_process.shutdown(Signal::SIGTERM).await?;
        }

        match llamacpp_applicable_state {
            Some(applicable_state) => {
                let llamacpp_process = LlamaCppProcess::new(
                    applicable_state,
                    self.llamacpp_listen_addr,
                    self.llamacpp_server_bin_path.clone(),
                )?;

                llamacpp_process.spawn().await?;

                self.llamacpp_process = Some(llamacpp_process);

                Ok(())
            }
            None => Ok(()),
        }
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
        let mut reconciled_state = self.llamacpp_applicable_state_holder.subscribe();
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
                    if let Some(llamacpp_process) = &self.llamacpp_process {
                        if let Err(err) = llamacpp_process.check_health().await {
                            error!("Unable to check health of llama-server: {err}");
                        }

                        if !llamacpp_process.is_healthy() {
                            error!("llama-server is unhealthy");
                        }
                    }
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
