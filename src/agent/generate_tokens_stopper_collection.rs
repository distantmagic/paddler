use anyhow::anyhow;
use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::mpsc;

pub struct GenerateTokensStopperCollection {
    generate_tokens_stoppers: DashMap<String, mpsc::UnboundedSender<()>>,
}

impl GenerateTokensStopperCollection {
    pub fn new() -> Self {
        Self {
            generate_tokens_stoppers: DashMap::new(),
        }
    }

    pub fn deregister_stopper(&self, request_id: String) -> Result<()> {
        if let Some(stopper) = self.generate_tokens_stoppers.remove(&request_id) {
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
        if self.generate_tokens_stoppers.contains_key(&request_id) {
            return Err(anyhow!(
                "Stopper for request_id {request_id} already exists"
            ));
        }

        self.generate_tokens_stoppers.insert(request_id, stopper);

        Ok(())
    }

    pub fn stop(&self, request_id: String) -> Result<()> {
        if let Some(stopper) = self.generate_tokens_stoppers.get(&request_id) {
            stopper.send(())?;

            Ok(())
        } else {
            Err(anyhow!("No stopper found for request_id {request_id}"))
        }
    }
}
