#![cfg(feature = "tests_that_use_llms")]

use hf_hub::Cache;
use llama_cpp_bindings::error::MarkerDetectionError;
use llama_cpp_bindings::gguf_context::GgufContext;
use tempfile::TempDir;
use tokio_util::sync::CancellationToken;

use paddler_agent_text_generation::multimodal_projection::MultimodalProjection;
use paddler_agent_text_generation::text_generation_error::TextGenerationError;
use paddler_agent_text_generation::text_generation_pipeline::TextGenerationPipeline;
use paddler_agent_text_generation::text_generation_settings::TextGenerationSettings;
use paddler_inference_parameters::model_runtime_parameters::ModelRuntimeParameters;
use paddler_inference_parameters::sampling_parameters::SamplingParameters;
use paddler_messaging::chat_template_source::ChatTemplateSource;
use paddler_messaging::multimodal_settings::MultimodalSettings;
use paddler_test_cluster_harness::model_card::qwen3_0_6b::qwen3_0_6b;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::first_message_content_chat_template::first_message_content_chat_template;

#[tokio::test(flavor = "multi_thread")]
async fn a_text_generation_pipeline_refuses_a_model_whose_embedded_template_hides_its_streaming_markers()
 {
    let model_directory = TempDir::new().expect("a temporary model directory must be creatable");
    let model_path = model_directory
        .path()
        .join("qwen3-with-an-embedded-template-that-does-not-parse.gguf");
    let mut gguf_context = GgufContext::from_file(
        qwen3_0_6b()
            .reference
            .cached_path(&Cache::from_env())
            .expect("Qwen3 must be in the Hugging Face cache"),
    )
    .expect("the Qwen3 GGUF must be readable");

    gguf_context.set_val_str(c"tokenizer.chat_template", c"{% if %}");
    gguf_context
        .write_to_file(&model_path)
        .expect("the model with a template that does not parse must be writable");

    assert_eq!(
        TextGenerationPipeline {
            inference_runtime_context: agent_inference_runtime_context(1),
            model_path,
            model_runtime_parameters: ModelRuntimeParameters::default(),
            text_generation_settings: TextGenerationSettings {
                chat_template_source: ChatTemplateSource::Override(
                    first_message_content_chat_template()
                ),
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
            TextGenerationError::StreamingMarkersUnavailable(
                MarkerDetectionError::ReasoningMarkerDetectionFailed {
                    message: "Unexpected token: %} of type 9".to_owned(),
                }
            )
            .to_string()
        )
    );
}
