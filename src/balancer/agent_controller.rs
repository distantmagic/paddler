use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::mpsc;

use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller_snapshot::AgentControllerSnapshot;
use crate::produces_snapshot::ProducesSnapshot;
use crate::sends_serialized_message::SendsSerializedMessage;

pub struct AgentController {
    pub agent_tx: mpsc::Sender<String>,
    pub id: String,
    pub name: Option<String>,
    pub slots_processing: AtomicValue,
    pub slots_total: i32,
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

#[async_trait]
impl SendsSerializedMessage for AgentController {
    async fn send_serialized<TMessage: Send + Serialize>(&self, message: TMessage) -> Result<()> {
        let serialized_message = serde_json::to_string(&message)?;

        self.agent_tx.send(serialized_message).await?;

        Ok(())
    }
}
