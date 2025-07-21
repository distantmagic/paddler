use std::thread;

use actix::Addr;
use anyhow::anyhow;
use anyhow::Result;
use tokio::sync::oneshot;

use crate::agent::llamacpp_slot::LlamaCppSlot;

pub struct LlamaCppArbiterController {
    pub llamacpp_slot_addr: Addr<LlamaCppSlot>,
    shutdown_tx: oneshot::Sender<()>,
    sync_arbiter_thread_handle: thread::JoinHandle<Result<()>>,
}

impl LlamaCppArbiterController {
    pub fn new(
        llamacpp_slot_addr: Addr<LlamaCppSlot>,
        shutdown_tx: oneshot::Sender<()>,
        sync_arbiter_thread_handle: thread::JoinHandle<Result<()>>,
    ) -> Self {
        Self {
            llamacpp_slot_addr,
            shutdown_tx,
            sync_arbiter_thread_handle,
        }
    }

    pub async fn shutdown(self) -> Result<()> {
        self.shutdown_tx
            .send(())
            .map_err(|err| anyhow!("Failed to send shutdown signal: {err:?}"))?;

        self.sync_arbiter_thread_handle
            .join()
            .map_err(|err| anyhow!("Failed to join sync arbiter thread: {err:?}"))??;

        Ok(())
    }
}
