#![cfg(feature = "tests_that_use_llms")]

use hf_hub::Cache;
use tokio_util::sync::CancellationToken;

use paddler_agent_text_generation::multimodal_projection::MultimodalProjection;
use paddler_agent_text_generation::text_generation_pipeline::TextGenerationPipeline;
use paddler_agent_text_generation::text_generation_settings::TextGenerationSettings;
use paddler_inference_parameters::model_runtime_parameters::ModelRuntimeParameters;
use paddler_inference_parameters::sampling_parameters::SamplingParameters;
use paddler_messaging::chat_template::ChatTemplate;
use paddler_messaging::chat_template_source::ChatTemplateSource;
use paddler_messaging::multimodal_settings::MultimodalSettings;
use paddler_test_cluster_harness::model_card::smolvlm2_256m::smolvlm2_256m;
use paddler_test_cluster_harness::model_card::smolvlm2_256m_mmproj::smolvlm2_256m_mmproj;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;

#[tokio::test(flavor = "multi_thread")]
async fn a_multimodal_pipeline_refuses_a_template_that_raises_while_probing_its_bos_token() {
    assert_eq!(
        TextGenerationPipeline {
            inference_runtime_context: agent_inference_runtime_context(1),
            model_path: smolvlm2_256m()
                .reference
                .cached_path(&Cache::from_env())
                .expect("the vision model must be in the Hugging Face cache"),
            model_runtime_parameters: ModelRuntimeParameters::default(),
            text_generation_settings: TextGenerationSettings {
                chat_template_source: ChatTemplateSource::Override(ChatTemplate {
                    content: "{{ raise_exception('the template refuses to render') }}".to_owned(),
                }),
                image_resize_to_fit: MultimodalSettings::default().image_resize_to_fit,
                multimodal_projection: MultimodalProjection::File(
                    smolvlm2_256m_mmproj()
                        .reference
                        .cached_path(&Cache::from_env())
                        .expect("the multimodal projection must be in the Hugging Face cache"),
                ),
                sampling_parameters: SamplingParameters::default(),
            },
        }
        .spawn(&CancellationToken::new())
        .await
        .err()
        .as_ref()
        .map(ToString::to_string),
        Some(
            "failed to render chat template: invalid operation: Model's chat template raised an exception: 'the template refuses to render' (in chat_template:1)"
                .to_owned()
        )
    );
}
