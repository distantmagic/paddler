use std::sync::RwLock;

use actix_web::web::Data;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::Message as AgentJsonRpcMessage;
use crate::agent::jsonrpc::Notification as AgentJsonRpcNotification;
use crate::agent::jsonrpc::Request as AgentJsonRpcRequest;
use crate::agent_desired_state::AgentDesiredState;
use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller_snapshot::AgentControllerSnapshot;
use crate::balancer::generate_tokens_controller::GenerateTokensController;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::jsonrpc::RequestEnvelope;
use crate::produces_snapshot::ProducesSnapshot;
use crate::request_params::GenerateTokensParams;
use crate::sends_rpc_message::SendsRpcMessage;
use crate::sets_desired_state::SetsDesiredState;

pub struct AgentController {
    pub agent_message_tx: mpsc::UnboundedSender<AgentJsonRpcMessage>,
    pub connection_close_rx: broadcast::Receiver<()>,
    pub desired_slots_total: AtomicValue,
    pub generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    pub id: String,
    pub model_path: RwLock<Option<String>>,
    pub name: Option<String>,
    pub slots_processing: AtomicValue,
    pub slots_total: AtomicValue,
}

impl AgentController {
    pub async fn generate_tokens(
        &self,
        request_id: String,
        generate_tokens_params: GenerateTokensParams,
    ) -> Result<GenerateTokensController> {
        let (generated_tokens_tx, generated_tokens_rx) = mpsc::unbounded_channel();

        self.generate_tokens_sender_collection
            .register_sender(request_id.clone(), generated_tokens_tx)?;

        self.send_rpc_message(AgentJsonRpcMessage::Request(RequestEnvelope {
            id: request_id.clone(),
            request: AgentJsonRpcRequest::GenerateTokens(generate_tokens_params.clone()),
        }))
        .await?;

        Ok(GenerateTokensController {
            generate_tokens_sender_collection: self.generate_tokens_sender_collection.clone(),
            generated_tokens_rx,
            request_id,
        })
    }

    pub fn set_model_path(&self, model_path: Option<String>) {
        let mut locked_path = self
            .model_path
            .write()
            .expect("Poisoned lock on model path");

        *locked_path = model_path;
    }

    pub async fn stop_generating_tokens(&self, request_id: String) -> Result<()> {
        self.send_rpc_message(AgentJsonRpcMessage::Notification(
            AgentJsonRpcNotification::StopGeneratingTokens(request_id),
        ))
        .await?;

        Ok(())
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
        self.agent_message_tx.send(message)?;

        Ok(())
    }
}

#[async_trait]
impl SetsDesiredState for AgentController {
    async fn set_desired_state(&self, desired_state: AgentDesiredState) -> Result<()> {
        self.send_rpc_message(AgentJsonRpcMessage::Notification(
            AgentJsonRpcNotification::SetState(SetStateParams { desired_state }),
        ))
        .await
    }
}
