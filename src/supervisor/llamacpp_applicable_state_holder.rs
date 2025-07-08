use anyhow::Result;
use tokio::sync::watch::channel;
use tokio::sync::watch::Receiver;
use tokio::sync::watch::Sender;

use crate::supervisor::llamacpp_applicable_state::LlamaCppApplicableState;

pub struct LlamaCppApplicableStateHolder {
    change_notifier: Sender<Option<LlamaCppApplicableState>>,
}

impl LlamaCppApplicableStateHolder {
    pub fn new() -> Self {
        let (change_notifier, _) = channel::<Option<LlamaCppApplicableState>>(None);

        Self {
            change_notifier,
        }
    }

    pub fn set_applicable_state(
        &self,
        applicable_state: Option<LlamaCppApplicableState>,
    ) -> Result<()> {
        Ok(self.change_notifier.send(applicable_state)?)
    }

    pub fn subscribe(&self) -> Receiver<Option<LlamaCppApplicableState>> {
        self.change_notifier.subscribe()
    }
}
