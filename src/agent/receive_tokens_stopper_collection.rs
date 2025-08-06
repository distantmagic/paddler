use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::mpsc;

use crate::agent::receive_tokens_stopper_drop_guard::ReceiveTokensStopperDropGuard;

pub struct ReceiveTokensStopperCollection {
    receive_tokens_stoppers: DashMap<String, mpsc::UnboundedSender<()>>,
}

impl ReceiveTokensStopperCollection {
    pub fn new() -> Self {
        Self {
            receive_tokens_stoppers: DashMap::new(),
        }
    }

    pub fn deregister_stopper(&self, request_id: String) -> Result<()> {
        if let Some(stopper) = self.receive_tokens_stoppers.remove(&request_id) {
            drop(stopper);

            Ok(())
        } else {
            Err(anyhow!("No stopper found for request_id {request_id}"))
        }
    }

    pub fn register_stopper(
        &self,
        request_id: String,
        stopper: mpsc::UnboundedSender<()>,
    ) -> Result<()> {
        if self.receive_tokens_stoppers.contains_key(&request_id) {
            return Err(anyhow!(
                "Stopper for request_id {request_id} already exists"
            ));
        }

        self.receive_tokens_stoppers.insert(request_id, stopper);

        Ok(())
    }

    pub fn register_stopper_with_guard(
        self: &Arc<Self>,
        request_id: String,
        stopper: mpsc::UnboundedSender<()>,
    ) -> Result<ReceiveTokensStopperDropGuard> {
        self.register_stopper(request_id.clone(), stopper)?;

        Ok(ReceiveTokensStopperDropGuard {
            receive_tokens_stopper_collection: self.clone(),
            request_id,
        })
    }

    pub fn stop(&self, request_id: String) -> Result<()> {
        if let Some(stopper) = self.receive_tokens_stoppers.get(&request_id) {
            stopper.send(())?;

            Ok(())
        } else {
            Err(anyhow!("No stopper found for request_id {request_id}"))
        }
    }
}
