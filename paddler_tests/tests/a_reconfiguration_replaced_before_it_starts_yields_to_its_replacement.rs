#![cfg(feature = "tests_that_use_llms")]

use std::collections::BTreeSet;

use paddler_agent::desired_state_reconciliation::DesiredStateReconciliation;
use paddler_inference_parameters::embedding_parameters::EmbeddingParameters;
use paddler_inference_parameters::pooling_type::PoolingType;
use paddler_messaging::agent_runtime_status::AgentRuntimeStatus;
use paddler_messaging::agent_state_application_status::AgentStateApplicationStatus;
use paddler_messaging::agent_status::AgentStatus;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_messaging::model_download_status::ModelDownloadStatus;
use paddler_messaging::produces_snapshot::ProducesSnapshot as _;
use paddler_tests::desired_state_with_chat_template_override::desired_state_with_chat_template_override;
use paddler_tests::first_message_content_chat_template::first_message_content_chat_template;
use paddler_tests::kev_0_8b_desired_state::kev_0_8b_desired_state;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::serving_pipeline_arbiter::ServingPipelineArbiter;

use crate::swappable_model_file::SwappableModelFile;

#[tokio::test(flavor = "multi_thread")]
async fn a_reconfiguration_replaced_before_it_starts_yields_to_its_replacement() {
    let swappable_model_file = SwappableModelFile::link_to(&kev_0_8b_target_path("model.gguf"));
    let served_desired_state = BalancerDesiredState {
        inference_mode: InferenceMode::TextGenerationAndEmbeddings,
        model: swappable_model_file.agent_desired_model(),
        ..kev_0_8b_desired_state()
    };
    let mut serving_pipeline_arbiter = ServingPipelineArbiter::new(1);

    assert_eq!(
        serving_pipeline_arbiter
            .serve(served_desired_state.clone())
            .await,
        DesiredStateReconciliation::Reconciled
    );

    swappable_model_file.replace_with_an_unloadable_file();

    assert_eq!(
        serving_pipeline_arbiter
            .apply(BalancerDesiredState {
                embeddings: EmbeddingParameters {
                    pooling_type: PoolingType::Mean,
                    ..served_desired_state.embeddings.clone()
                },
                ..served_desired_state.clone()
            })
            .await,
        DesiredStateReconciliation::Reconciled
    );
    assert_eq!(
        serving_pipeline_arbiter
            .serve(desired_state_with_chat_template_override(
                served_desired_state,
                first_message_content_chat_template(),
            ))
            .await,
        DesiredStateReconciliation::Reconciled
    );
    assert_eq!(
        serving_pipeline_arbiter
            .inference_runtime_context
            .slot_aggregated_status
            .make_snapshot()
            .status,
        AgentStatus {
            desired_slots_total: 1,
            download_status: ModelDownloadStatus::NotDownloading,
            issues: BTreeSet::new(),
            model_path: Some(swappable_model_file.model_path().display().to_string()),
            runtime: AgentRuntimeStatus::Serving {
                inference_mode: InferenceMode::TextGenerationAndEmbeddings,
                slots_total: 1,
            },
            state_application_status: AgentStateApplicationStatus::Applied,
            uses_chat_template_override: true,
        }
    );

    serving_pipeline_arbiter.shut_down().await;
}
