use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::Message as AgentJsonRpcMessage;
use crate::agent::jsonrpc::Notification as AgentJsonRpcNotification;
use crate::agent::llamacpp_desired_state::LlamaCppDesiredState;
use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller_snapshot::AgentControllerSnapshot;
use crate::produces_snapshot::ProducesSnapshot;
use crate::sends_rpc_message::SendsRpcMessage;

pub struct AgentController {
    pub agent_tx: mpsc::Sender<AgentJsonRpcMessage>,
    pub id: String,
    pub name: Option<String>,
    pub slots_processing: AtomicValue,
    pub slots_total: i32,
}

impl AgentController {
    pub async fn set_desired_state(&self, desired_state: LlamaCppDesiredState) -> Result<()> {
        self.send_rpc_message(AgentJsonRpcMessage::Notification(
            AgentJsonRpcNotification::SetState(SetStateParams {
                desired_state,
            }),
        ))
        .await
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

#[async_trait]
impl SendsRpcMessage for AgentController {
    type Message = AgentJsonRpcMessage;

    async fn send_rpc_message(&self, message: Self::Message) -> Result<()> {
        self.agent_tx.send(message).await?;

        Ok(())
    }
}
