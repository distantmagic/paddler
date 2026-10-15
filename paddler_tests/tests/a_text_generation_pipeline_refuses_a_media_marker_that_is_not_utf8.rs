#![cfg(feature = "tests_that_use_llms")]

use hf_hub::Cache;
use tokio_util::sync::CancellationToken;

use paddler_agent_text_generation::multimodal_projection::MultimodalProjection;
use paddler_agent_text_generation::provides_media_marker::ProvidesMediaMarker;
use paddler_agent_text_generation::text_generation_error::TextGenerationError;
use paddler_agent_text_generation::text_generation_pipeline::TextGenerationPipeline;
use paddler_agent_text_generation::text_generation_settings::TextGenerationSettings;
use paddler_inference_parameters::model_runtime_parameters::ModelRuntimeParameters;
use paddler_inference_parameters::sampling_parameters::SamplingParameters;
use paddler_messaging::chat_template_source::ChatTemplateSource;
use paddler_messaging::multimodal_settings::MultimodalSettings;
use paddler_test_cluster_harness::model_card::qwen3_0_6b::qwen3_0_6b;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::media_marker_that_is_not_utf8::MediaMarkerThatIsNotUtf8;

#[tokio::test(flavor = "multi_thread")]
async fn a_text_generation_pipeline_refuses_a_media_marker_that_is_not_utf8() {
    assert_eq!(
        TextGenerationPipeline {
            inference_runtime_context: agent_inference_runtime_context(1),
            media_marker: MediaMarkerThatIsNotUtf8,
            model_path: qwen3_0_6b()
                .reference
                .cached_path(&Cache::from_env())
                .expect("Qwen3 must be in the Hugging Face cache"),
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
            TextGenerationError::MediaMarkerUnavailable(
                MediaMarkerThatIsNotUtf8
                    .media_marker()
                    .expect_err("the fake media marker must not be UTF-8")
            )
            .to_string()
        )
    );
}
