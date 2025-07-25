use std::sync::atomic::AtomicI32;
use std::sync::RwLock;

use actix_web::web::Data;
use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::Message as AgentJsonRpcMessage;
use crate::agent::jsonrpc::Notification as AgentJsonRpcNotification;
use crate::agent::jsonrpc::Request as AgentJsonRpcRequest;
use crate::agent_desired_state::AgentDesiredState;
use crate::atomic_value::AtomicValue;
use crate::balancer::agent_controller_snapshot::AgentControllerSnapshot;
use crate::balancer::agent_controller_update_result::AgentControllerUpdateResult;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::receive_tokens_controller::ReceiveTokensController;
use crate::jsonrpc::RequestEnvelope;
use crate::produces_snapshot::ProducesSnapshot;
use crate::request_params::ContinueConversationParams;
use crate::request_params::GenerateTokensParams;
use crate::sends_rpc_message::SendsRpcMessage;
use crate::sets_desired_state::SetsDesiredState;
use crate::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;

pub struct AgentController {
    pub agent_message_tx: mpsc::UnboundedSender<AgentJsonRpcMessage>,
    pub connection_close_rx: broadcast::Receiver<()>,
    pub desired_slots_total: AtomicValue<AtomicI32>,
    pub generate_tokens_sender_collection: Data<GenerateTokensSenderCollection>,
    pub id: String,
    pub model_path: RwLock<Option<String>>,
    pub name: Option<String>,
    pub newest_update_version: AtomicValue<AtomicI32>,
    pub slots_processing: AtomicValue<AtomicI32>,
    pub slots_total: AtomicValue<AtomicI32>,
}

impl AgentController {
    pub async fn continue_conversation(
        &self,
        request_id: String,
        continue_conversation_params: ContinueConversationParams,
    ) -> Result<ReceiveTokensController> {
        self.receiver_from_message(
            request_id.clone(),
            AgentJsonRpcMessage::Request(RequestEnvelope {
                id: request_id,
                request: AgentJsonRpcRequest::ContinueConversation(
                    continue_conversation_params.clone(),
                ),
            }),
        )
        .await
    }

    pub async fn generate_tokens(
        &self,
        request_id: String,
        generate_tokens_params: GenerateTokensParams,
    ) -> Result<ReceiveTokensController> {
        self.receiver_from_message(
            request_id.clone(),
            AgentJsonRpcMessage::Request(RequestEnvelope {
                id: request_id,
                request: AgentJsonRpcRequest::GenerateTokens(generate_tokens_params.clone()),
            }),
        )
        .await
    }

    pub fn get_model_path(&self) -> Option<String> {
        self.model_path
            .read()
            .expect("Poisoned lock on model path")
            .clone()
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

    pub fn update_from_slot_aggregated_status_snapshot(
        &self,
        SlotAggregatedStatusSnapshot {
            desired_slots_total,
            model_path,
            slots_processing,
            slots_total,
            version,
        }: SlotAggregatedStatusSnapshot,
    ) -> AgentControllerUpdateResult {
        if version < self.newest_update_version.get() {
            debug!("Discarding update with older version: {version}");

            return AgentControllerUpdateResult::NoMeaningfulChanges;
        }

        let mut changed = false;

        changed = changed || self.desired_slots_total.set_check(desired_slots_total);
        changed = changed || self.slots_processing.set_check(slots_processing);
        changed = changed || self.slots_total.set_check(slots_total);

        self.newest_update_version.set(version);

        if model_path != self.get_model_path() {
            changed = true;

            self.set_model_path(model_path);
        }

        if changed {
            AgentControllerUpdateResult::Updated
        } else {
            AgentControllerUpdateResult::NoMeaningfulChanges
        }
    }

    async fn receiver_from_message(
        &self,
        request_id: String,
        message: AgentJsonRpcMessage,
    ) -> Result<ReceiveTokensController> {
        let (generated_tokens_tx, generated_tokens_rx) = mpsc::unbounded_channel();

        self.generate_tokens_sender_collection
            .register_sender(request_id.clone(), generated_tokens_tx)?;
        self.send_rpc_message(message).await?;

        Ok(ReceiveTokensController {
            generate_tokens_sender_collection: self.generate_tokens_sender_collection.clone(),
            generated_tokens_rx,
            request_id,
        })
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
