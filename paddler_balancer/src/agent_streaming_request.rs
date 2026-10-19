use std::num::NonZeroU16;
use std::sync::Arc;

use paddler_messaging::decision_result::DecisionResult;
use paddler_messaging::embedding_result::EmbeddingResult;
use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_messaging::inference_capability::InferenceCapability;
use paddler_messaging::request_params::continue_from_conversation_history_params::ContinueFromConversationHistoryParams;
use paddler_messaging::request_params::continue_from_conversation_history_params::tool::tool_params::function_call::parameters_schema::validated_parameters_schema::ValidatedParametersSchema;
use paddler_messaging::request_params::continue_from_raw_prompt_params::ContinueFromRawPromptParams;
use paddler_messaging::request_params::decide_params::DecideParams;
use paddler_messaging::request_params::generate_embedding_batch_params::GenerateEmbeddingBatchParams;

use crate::agent_response_senders::AgentResponseSenders;
use crate::dispatch_requirement::DispatchRequirement;
use crate::into_reserved_agent_request::IntoReservedAgentRequest;
use crate::response_senders::ResponseSenders;
use crate::slot_demand::SlotDemand;

pub trait AgentStreamingRequest: IntoReservedAgentRequest {
    const INFERENCE_CAPABILITY: InferenceCapability;

    type Response: Send + 'static;

    fn response_senders(
        agent_response_senders: &AgentResponseSenders,
    ) -> &Arc<ResponseSenders<Self::Response>>;

    fn slot_demand(&self) -> SlotDemand;

    fn dispatch_requirement(&self) -> DispatchRequirement {
        DispatchRequirement {
            inference_capability: Self::INFERENCE_CAPABILITY,
            slot_demand: self.slot_demand(),
        }
    }
}

impl AgentStreamingRequest for ContinueFromConversationHistoryParams<ValidatedParametersSchema> {
    const INFERENCE_CAPABILITY: InferenceCapability = InferenceCapability::TextGeneration;

    type Response = GeneratedTokenResult;

    fn response_senders(
        agent_response_senders: &AgentResponseSenders,
    ) -> &Arc<ResponseSenders<Self::Response>> {
        &agent_response_senders.generated_tokens
    }

    fn slot_demand(&self) -> SlotDemand {
        SlotDemand::Exactly(NonZeroU16::MIN)
    }
}

impl AgentStreamingRequest for ContinueFromRawPromptParams {
    const INFERENCE_CAPABILITY: InferenceCapability = InferenceCapability::TextGeneration;

    type Response = GeneratedTokenResult;

    fn response_senders(
        agent_response_senders: &AgentResponseSenders,
    ) -> &Arc<ResponseSenders<Self::Response>> {
        &agent_response_senders.generated_tokens
    }

    fn slot_demand(&self) -> SlotDemand {
        SlotDemand::Exactly(NonZeroU16::MIN)
    }
}

impl AgentStreamingRequest for DecideParams {
    const INFERENCE_CAPABILITY: InferenceCapability = InferenceCapability::Decisions;

    type Response = DecisionResult;

    fn response_senders(
        agent_response_senders: &AgentResponseSenders,
    ) -> &Arc<ResponseSenders<Self::Response>> {
        &agent_response_senders.decision
    }

    fn slot_demand(&self) -> SlotDemand {
        SlotDemand::Exactly(self.required_slots())
    }
}

impl AgentStreamingRequest for GenerateEmbeddingBatchParams {
    const INFERENCE_CAPABILITY: InferenceCapability = InferenceCapability::Embeddings;

    type Response = EmbeddingResult;

    fn response_senders(
        agent_response_senders: &AgentResponseSenders,
    ) -> &Arc<ResponseSenders<Self::Response>> {
        &agent_response_senders.embedding
    }

    fn slot_demand(&self) -> SlotDemand {
        SlotDemand::UpTo(u16::try_from(self.input_batch.len()).unwrap_or(u16::MAX))
    }
}
