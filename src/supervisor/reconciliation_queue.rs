use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::errors::result::Result;
use crate::supervisor::change_request::ChangeRequest;

pub struct ReconciliationQueue {
    change_requests_receiver: Mutex<Receiver<ChangeRequest>>,
    change_requests_sender: Sender<ChangeRequest>,
}

impl ReconciliationQueue {
    pub fn new() -> Result<Self> {
        let (change_requests_sender, change_requests_receiver) = mpsc::channel(100);

        Ok(ReconciliationQueue {
            change_requests_receiver: Mutex::new(change_requests_receiver),
            change_requests_sender,
        })
    }

    pub async fn next_change_request(&self) -> Result<ChangeRequest> {
        let mut receiver = self.change_requests_receiver.lock().await;

        match receiver.recv().await {
            Some(change_request) => Ok(change_request),
            None => Err("No change request available".into()),
        }
    }

    pub async fn send_change_request(&self, change_request: ChangeRequest) -> Result<()> {
        Ok(self.change_requests_sender.send(change_request).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::supervisor::change_request::ChangeRequest;

    #[tokio::test]
    async fn test_reconciliation_queue() -> Result<()> {
        let queue = ReconciliationQueue::new()?;

        let change_request = ChangeRequest {};

        queue.send_change_request(change_request.clone()).await?;

        let received_request = queue.next_change_request().await?;

        assert_eq!(change_request, received_request);

        Ok(())
    }
}
