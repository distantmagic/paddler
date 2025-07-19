use anyhow::Result;
use tokio::sync::watch::channel;
use tokio::sync::watch::Receiver;
use tokio::sync::watch::Sender;

use crate::agent_applicable_state::AgentApplicableState;

pub struct AgentApplicableStateHolder {
    change_notifier: Sender<Option<AgentApplicableState>>,
}

impl AgentApplicableStateHolder {
    pub fn new() -> Self {
        let (change_notifier, _) = channel::<Option<AgentApplicableState>>(None);

        Self {
            change_notifier,
        }
    }

    pub fn set_applicable_state(
        &self,
        applicable_state: Option<AgentApplicableState>,
    ) -> Result<()> {
        Ok(self.change_notifier.send(applicable_state)?)
    }

    pub fn subscribe(&self) -> Receiver<Option<AgentApplicableState>> {
        self.change_notifier.subscribe()
    }
}
