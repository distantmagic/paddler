use tokio::sync::watch::channel;
use tokio::sync::watch::Receiver;
use tokio::sync::watch::Sender;

use crate::supervisor::llamacpp_applicable_state::LlamaCppApplicableState;

pub struct LlamaCppReconciledStateHolder {
    change_notifier: Sender<Option<LlamaCppApplicableState>>,
}

impl LlamaCppReconciledStateHolder {
    pub fn new() -> Self {
        let (change_notifier, _) = channel::<Option<LlamaCppApplicableState>>(None);

        Self {
            change_notifier,
        }
    }

    pub fn subscribe(&self) -> Receiver<Option<LlamaCppApplicableState>> {
        self.change_notifier.subscribe()
    }
}
