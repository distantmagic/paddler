#![cfg(feature = "tests_that_use_llms")]

use tokio_util::sync::CancellationToken;

use paddler_agent::inference_capabilities::InferenceCapabilities;
use paddler_agent::model_serving_settings::ModelServingSettings;
use paddler_agent::serving_startup_plan::ServingStartupPlan;
use paddler_agent_text_generation::text_generation_request::TextGenerationRequest;
use paddler_inference_parameters::pooling_type::PoolingType;
use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_tests::raw_prompt_generation::RawPromptGeneration;

use crate::kev_model_serving_every_capability::KevModelServingEveryCapability;

#[tokio::test(flavor = "multi_thread")]
async fn a_released_restart_plan_stops_a_kept_scheduler_it_alone_holds() {
    let KevModelServingEveryCapability {
        llama_runtime,
        model_serving_settings,
        served_model,
    } = KevModelServingEveryCapability::start(&CancellationToken::new()).await;
    let text_generation_request_preparer = served_model
        .capabilities
        .text_generation
        .request_preparer
        .clone();
    let restart_plan = served_model.plan_restart(ModelServingSettings {
        capabilities: InferenceCapabilities {
            embeddings: PoolingType::Mean,
            ..model_serving_settings.capabilities.clone()
        },
        ..model_serving_settings
    });

    served_model.shut_down().await;
    ServingStartupPlan::OnLoadedModel(restart_plan)
        .release()
        .await;

    let RawPromptGeneration {
        agent_request,
        generate_tokens_stop_tx: _generate_tokens_stop_tx,
        mut generated_tokens_rx,
    } = RawPromptGeneration::new(
        "The capital of France is",
        llama_runtime
            .inference_runtime_context
            .slot_aggregated_status
            .clone(),
    );

    text_generation_request_preparer
        .prepare(TextGenerationRequest::ContinueFromRawPrompt(agent_request));

    assert_eq!(
        generated_tokens_rx.recv().await,
        Some(GeneratedTokenResult::SchedulerUnavailable(
            "agent: the scheduler is no longer accepting requests".to_owned()
        ))
    );
}
