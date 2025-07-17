use anyhow::Result;
use tokio::sync::mpsc;

use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::Notification;
use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;
use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller_snapshot::AgentControllerSnapshot;
use crate::produces_snapshot::ProducesSnapshot;

pub struct AgentController {
    pub id: String,
    pub name: Option<String>,
    pub session_tx: mpsc::Sender<String>,
    pub slots_processing: AtomicValue,
    pub slots_total: i32,
}

impl AgentController {
    pub async fn set_desired_state(&self, desired_state: LlamaCppDesiredState) -> Result<()> {
        let state_json = serde_json::to_string(&Notification::SetState(SetStateParams {
            desired_state,
        }))?;

        self.session_tx.send(state_json).await?;

        Ok(())
    }
}

impl ProducesSnapshot for AgentController {
    type Snapshot = AgentControllerSnapshot;

    fn make_snapshot(&self) -> Self::Snapshot {
        AgentControllerSnapshot {
            id: self.id.clone(),
            name: self.name.clone(),
            slots_processing: self.slots_processing.get(),
            slots_total: self.slots_total,
        }
    }
}
