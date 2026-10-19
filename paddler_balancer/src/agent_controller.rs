use std::num::NonZeroU16;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use parking_lot::RwLock;
use tokio_util::sync::CancellationToken;

use paddler_messaging::agent_controller_snapshot::AgentControllerSnapshot;
use paddler_messaging::agent_desired_state::AgentDesiredState;
use paddler_messaging::atomic_value::AtomicValue;
use paddler_messaging::chat_template::ChatTemplate;
use paddler_messaging::jsonrpc::request_envelope::RequestEnvelope;
use paddler_messaging::management_socket::agent::message::Message as AgentJsonRpcMessage;
use paddler_messaging::management_socket::agent::notification::Notification as AgentJsonRpcNotification;
use paddler_messaging::management_socket::agent::request::Request as AgentJsonRpcRequest;
use paddler_messaging::model_metadata::ModelMetadata;
use paddler_messaging::produces_snapshot::ProducesSnapshot;
use paddler_messaging::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;

use crate::accept_newer_status_report::accept_newer_status_report;
use crate::agent_controller_update_result::AgentControllerUpdateResult;
use crate::agent_desired_state_notification::agent_desired_state_notification;
use crate::agent_outbox::AgentOutbox;
use crate::agent_response_receiver::AgentResponseReceiver;
use crate::agent_response_senders::AgentResponseSenders;
use crate::agent_streaming_request::AgentStreamingRequest;
use crate::response_senders::ResponseSenders;

pub struct AgentController {
    pub agent_outbox: AgentOutbox,
    pub agent_response_senders: AgentResponseSenders,
    pub connection_close: CancellationToken,
    pub id: String,
    pub name: Option<String>,
    pub reported_status: RwLock<SlotAggregatedStatusSnapshot>,
    pub slots_processing: AtomicValue<AtomicU64>,
}

impl AgentController {
    #[must_use]
    pub fn get_chat_template_override(&self) -> AgentResponseReceiver<Option<ChatTemplate>> {
        self.relay_request(
            AgentJsonRpcRequest::GetChatTemplateOverride,
            &self.agent_response_senders.chat_template_override,
        )
    }

    #[must_use]
    pub fn get_model_metadata(&self) -> AgentResponseReceiver<Option<ModelMetadata>> {
        self.relay_request(
            AgentJsonRpcRequest::GetModelMetadata,
            &self.agent_response_senders.model_metadata,
        )
    }

    pub fn set_desired_state(&self, desired_state: AgentDesiredState) {
        self.agent_outbox
            .push(agent_desired_state_notification(desired_state));
    }

    pub fn stop_responding_to(&self, request_id: String) {
        self.agent_outbox.push(AgentJsonRpcMessage::Notification(
            AgentJsonRpcNotification::StopRespondingTo(request_id),
        ));
    }

    #[must_use]
    pub fn stream_responses_to<TRequest: AgentStreamingRequest>(
        &self,
        request: TRequest,
        reserved_slots: NonZeroU16,
    ) -> AgentResponseReceiver<TRequest::Response> {
        self.relay_request(
            request.into_reserved_agent_request(reserved_slots),
            TRequest::response_senders(&self.agent_response_senders),
        )
    }

    pub fn update_from_slot_aggregated_status_snapshot(
        &self,
        reported_snapshot: SlotAggregatedStatusSnapshot,
    ) -> AgentControllerUpdateResult {
        accept_newer_status_report(&mut self.reported_status.write(), reported_snapshot)
    }

    fn relay_request<TResponse>(
        &self,
        request: AgentJsonRpcRequest,
        response_senders: &Arc<ResponseSenders<TResponse>>,
    ) -> AgentResponseReceiver<TResponse> {
        let agent_response_receiver = AgentResponseReceiver::register(response_senders);

        self.agent_outbox
            .push(AgentJsonRpcMessage::Request(RequestEnvelope {
                id: agent_response_receiver
                    .request_registration
                    .request_id
                    .clone(),
                request,
            }));

        agent_response_receiver
    }
}

impl ProducesSnapshot for AgentController {
    type Snapshot = AgentControllerSnapshot;

    fn make_snapshot(&self) -> Self::Snapshot {
        AgentControllerSnapshot {
            id: self.id.clone(),
            name: self.name.clone(),
            slots_processing: self.slots_processing.get(),
            status: self.reported_status.read().status.clone(),
        }
    }
}
