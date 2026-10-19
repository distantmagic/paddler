use paddler_agent_runtime::loaded_llama_model::LoadedLlamaModel;
use paddler_agent_runtime::shuts_down_unless_shared::ShutsDownUnlessShared as _;
use paddler_agent_status::serving_report::ServingReport;
use paddler_agent_text_generation::text_generation_request::TextGenerationRequest;

use crate::capability_plan::CapabilityPlan;
use crate::inference_capabilities::InferenceCapabilities;
use crate::loaded_model_serving_plan::LoadedModelServingPlan;
use crate::model_serving_settings::ModelServingSettings;
use crate::pipeline_request::PipelineRequest;
use crate::served_capabilities::ServedCapabilities;

pub struct ServedModel {
    pub capabilities: ServedCapabilities,
    pub loaded_llama_model: LoadedLlamaModel,
    pub model_serving_settings: ModelServingSettings,
}

impl ServedModel {
    pub fn forward(&self, agent_name: Option<&str>, request: PipelineRequest) {
        match request {
            PipelineRequest::ContinueFromConversationHistory(request) => {
                self.capabilities.text_generation.request_preparer.prepare(
                    TextGenerationRequest::ContinueFromConversationHistory(request),
                );
            }
            PipelineRequest::ContinueFromRawPrompt(request) => {
                self.capabilities
                    .text_generation
                    .request_preparer
                    .prepare(TextGenerationRequest::ContinueFromRawPrompt(request));
            }
            PipelineRequest::Decide(request) => {
                self.capabilities.decisions.forward(agent_name, request);
            }
            PipelineRequest::GenerateEmbeddingBatch(request) => {
                self.capabilities
                    .embeddings
                    .request_preparer
                    .prepare(request);
            }
        }
    }

    #[must_use]
    pub fn plan_restart(&self, desired_settings: ModelServingSettings) -> LoadedModelServingPlan {
        let ServedCapabilities {
            decisions: served_decisions,
            embeddings: served_embeddings,
            text_generation: served_text_generation,
        } = &self.capabilities;
        let served_settings = &self.model_serving_settings;
        let capability_plans = if served_settings.model_runtime_parameters
            == desired_settings.model_runtime_parameters
        {
            InferenceCapabilities {
                decisions: CapabilityPlan::between(
                    served_decisions,
                    &served_settings.capabilities.decisions,
                    &desired_settings.capabilities.decisions,
                ),
                embeddings: CapabilityPlan::between(
                    served_embeddings,
                    &served_settings.capabilities.embeddings,
                    &desired_settings.capabilities.embeddings,
                ),
                text_generation: CapabilityPlan::between(
                    served_text_generation,
                    &served_settings.capabilities.text_generation,
                    &desired_settings.capabilities.text_generation,
                ),
            }
        } else {
            InferenceCapabilities {
                decisions: CapabilityPlan::Start,
                embeddings: CapabilityPlan::Start,
                text_generation: CapabilityPlan::Start,
            }
        };

        LoadedModelServingPlan {
            capability_plans,
            loaded_llama_model: self.loaded_llama_model.clone(),
            model_serving_settings: desired_settings,
        }
    }

    #[must_use]
    pub fn serving_report(&self, uses_chat_template_override: bool) -> ServingReport {
        ServingReport {
            inference_mode: self.model_serving_settings.inference_mode(),
            model_path: self
                .loaded_llama_model
                .llama_model_settings
                .reported_model_path()
                .model_path,
            serving_issues: self
                .capabilities
                .text_generation
                .request_preparer
                .preparation
                .conversation_support
                .serving_issues(),
            uses_chat_template_override,
        }
    }

    pub async fn shut_down(self) {
        let Self {
            capabilities,
            loaded_llama_model,
            ..
        } = self;

        capabilities.shut_down_unless_shared().await;
        loaded_llama_model.release().await;
    }
}
