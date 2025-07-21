use std::sync::RwLock;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::Message as AgentJsonRpcMessage;
use crate::agent::jsonrpc::Notification as AgentJsonRpcNotification;
use crate::agent_desired_state::AgentDesiredState;
use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller_snapshot::AgentControllerSnapshot;
use crate::produces_snapshot::ProducesSnapshot;
use crate::sends_rpc_message::SendsRpcMessage;
use crate::sets_desired_state::SetsDesiredState;

pub struct AgentController {
    pub agent_tx: mpsc::Sender<AgentJsonRpcMessage>,
    pub desired_slots_total: AtomicValue,
    pub id: String,
    pub model_path: RwLock<Option<String>>,
    pub name: Option<String>,
    pub slots_processing: AtomicValue,
    pub slots_total: AtomicValue,
}

impl AgentController {
    pub fn set_model_path(&self, model_path: Option<String>) {
        let mut locked_path = self
            .model_path
            .write()
            .expect("Poisoned lock on model path");

        *locked_path = model_path;
    }
}

impl ProducesSnapshot for AgentController {
    type Snapshot = AgentControllerSnapshot;

    fn make_snapshot(&self) -> Self::Snapshot {
        AgentControllerSnapshot {
            desired_slots_total: self.desired_slots_total.get(),
            id: self.id.clone(),
            model_path: self
                .model_path
                .read()
                .expect("Poisoned lock on model path")
                .clone(),
            name: self.name.clone(),
            slots_processing: self.slots_processing.get(),
            slots_total: self.slots_total.get(),
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

#[async_trait]
impl SetsDesiredState for AgentController {
    async fn set_desired_state(&self, desired_state: AgentDesiredState) -> Result<()> {
        self.send_rpc_message(AgentJsonRpcMessage::Notification(
            AgentJsonRpcNotification::SetState(SetStateParams {
                desired_state,
            }),
        ))
        .await
    }
}
