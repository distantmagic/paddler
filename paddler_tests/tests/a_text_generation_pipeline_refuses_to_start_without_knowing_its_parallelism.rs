use std::io;
use std::path::PathBuf;

use tokio_util::sync::CancellationToken;

use paddler_agent_runtime::agent_runtime_error::AgentRuntimeError;
use paddler_agent_text_generation::multimodal_projection::MultimodalProjection;
use paddler_agent_text_generation::text_generation_error::TextGenerationError;
use paddler_agent_text_generation::text_generation_pipeline::TextGenerationPipeline;
use paddler_agent_text_generation::text_generation_settings::TextGenerationSettings;
use paddler_inference_parameters::model_runtime_parameters::ModelRuntimeParameters;
use paddler_inference_parameters::sampling_parameters::SamplingParameters;
use paddler_messaging::chat_template_source::ChatTemplateSource;
use paddler_messaging::multimodal_settings::MultimodalSettings;
use paddler_tests::inference_runtime_context_with_unknown_parallelism::inference_runtime_context_with_unknown_parallelism;

#[tokio::test(flavor = "multi_thread")]
async fn a_text_generation_pipeline_refuses_to_start_without_knowing_its_parallelism() {
    assert_eq!(
        TextGenerationPipeline {
            inference_runtime_context: inference_runtime_context_with_unknown_parallelism(1),
            model_path: PathBuf::from("model-that-is-never-read.gguf"),
            model_runtime_parameters: ModelRuntimeParameters::default(),
            text_generation_settings: TextGenerationSettings {
                chat_template_source: ChatTemplateSource::EmbeddedInModel,
                image_resize_to_fit: MultimodalSettings::default().image_resize_to_fit,
                multimodal_projection: MultimodalProjection::NotConfigured,
                sampling_parameters: SamplingParameters::default(),
            },
        }
        .spawn(&CancellationToken::new())
        .await
        .err()
        .as_ref()
        .map(ToString::to_string),
        Some(
            TextGenerationError::AgentRuntime(AgentRuntimeError::AvailableParallelismUnknown(
                io::Error::other("the machine hides how many threads it runs in parallel")
            ))
            .to_string()
        )
    );
}
