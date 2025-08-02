use anyhow::Result;
use tokio::sync::watch;

use crate::holds_applicable_state::HoldsApplicableState;
use crate::agent_applicable_state::AgentApplicableState;

pub struct AgentApplicableStateHolder {
    change_notifier: watch::Sender<Option<AgentApplicableState>>,
}

impl AgentApplicableStateHolder {
    pub fn new() -> Self {
        let (change_notifier, _) = watch::channel::<Option<AgentApplicableState>>(None);

        Self { change_notifier }
    }
}

impl HoldsApplicableState for AgentApplicableStateHolder {
    type ApplicableState = AgentApplicableState;

    fn set_applicable_state(
        &self,
        applicable_state: Option<AgentApplicableState>,
    ) -> Result<()> {
        Ok(self.change_notifier.send(applicable_state)?)
    }

    fn subscribe(&self) -> watch::Receiver<Option<AgentApplicableState>> {
        self.change_notifier.subscribe()
    }
}
