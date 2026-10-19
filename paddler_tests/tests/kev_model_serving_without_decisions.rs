#![cfg(feature = "tests_that_use_llms")]

use std::num::NonZeroU32;

use tokio_util::sync::CancellationToken;

use paddler_agent::applicable_decisions::ApplicableDecisions;
use paddler_agent::capability_plan::CapabilityPlan;
use paddler_agent::capability_starter::CapabilityStarter;
use paddler_agent::inference_capabilities::InferenceCapabilities;
use paddler_agent::loaded_model_serving_plan::LoadedModelServingPlan;
use paddler_agent::model_serving_settings::ModelServingSettings;
use paddler_agent::prepared_decisions::PreparedDecisions;
use paddler_agent::prepared_decisions_plan::PreparedDecisionsPlan;
use paddler_agent::served_model::ServedModel;
use paddler_agent_text_generation::multimodal_projection::MultimodalProjection;
use paddler_agent_text_generation::text_generation_settings::TextGenerationSettings;
use paddler_inference_parameters::pooling_type::PoolingType;
use paddler_inference_parameters::sampling_parameters::SamplingParameters;
use paddler_messaging::chat_template_source::ChatTemplateSource;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;

pub struct KevModelServingWithoutDecisions {
    pub llama_runtime: LlamaRuntime,
    pub model_serving_settings: ModelServingSettings,
    pub served_model: ServedModel,
}

impl KevModelServingWithoutDecisions {
    pub async fn start(agent_shutdown: &CancellationToken) -> Self {
        let model_path = kev_0_8b_target_path("model.gguf");
        let llama_runtime = LlamaRuntime::load(&model_path, agent_inference_runtime_context(1))
            .expect("the model must load");
        let model_serving_settings = ModelServingSettings {
            capabilities: InferenceCapabilities {
                decisions: ApplicableDecisions::Disabled,
                embeddings: PoolingType::Last,
                text_generation: TextGenerationSettings {
                    chat_template_source: ChatTemplateSource::EmbeddedInModel,
                    image_resize_to_fit: NonZeroU32::MIN,
                    multimodal_projection: MultimodalProjection::NotConfigured,
                    sampling_parameters: SamplingParameters::default(),
                },
            },
            model_path,
            model_runtime_parameters: llama_runtime.model_runtime_parameters.clone(),
        };
        let served_model = CapabilityStarter {
            cancellation_token: agent_shutdown,
            inference_runtime_context: &llama_runtime.inference_runtime_context,
        }
        .serve(LoadedModelServingPlan {
            capability_plans: InferenceCapabilities {
                decisions: PreparedDecisionsPlan::Start(PreparedDecisions::Disabled),
                embeddings: CapabilityPlan::Start,
                text_generation: CapabilityPlan::Start,
            },
            loaded_llama_model: llama_runtime.loaded_llama_model.clone(),
            model_serving_settings: model_serving_settings.clone(),
        })
        .await
        .expect("text generation and embeddings must start");

        Self {
            llama_runtime,
            model_serving_settings,
            served_model,
        }
    }
}
