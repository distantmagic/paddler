use anyhow::anyhow;
use anyhow::Result;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::llamacpp::llamacpp_state::LlamaCppState;

const RECONCILIATION_QUEUE_BUFFER_SIZE: usize = 100;

pub struct ReconciliationQueue {
    change_requests_receiver: Mutex<Receiver<LlamaCppState>>,
    change_requests_sender: Sender<LlamaCppState>,
}

impl ReconciliationQueue {
    pub fn new() -> Result<Self> {
        let (change_requests_sender, change_requests_receiver) =
            mpsc::channel(RECONCILIATION_QUEUE_BUFFER_SIZE);

        Ok(ReconciliationQueue {
            change_requests_receiver: Mutex::new(change_requests_receiver),
            change_requests_sender,
        })
    }

    pub async fn next_change_request(&self) -> Result<LlamaCppState> {
        let mut receiver = self.change_requests_receiver.lock().await;

        match receiver.recv().await {
            Some(desired_state) => Ok(desired_state),
            None => Err(anyhow!("No change request available")),
        }
    }

    pub async fn register_change_request(&self, desired_state: LlamaCppState) -> Result<()> {
        Ok(self.change_requests_sender.send(desired_state).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llamacpp::llamacpp_state::LlamaCppState;

    #[tokio::test]
    async fn test_reconciliation_queue() -> Result<()> {
        let queue = ReconciliationQueue::new()?;

        let desired_state = LlamaCppState {
            is_running: true,
        };

        queue.register_change_request(desired_state.clone()).await?;

        let received_request = queue.next_change_request().await?;

        assert_eq!(desired_state, received_request);

        Ok(())
    }
}
