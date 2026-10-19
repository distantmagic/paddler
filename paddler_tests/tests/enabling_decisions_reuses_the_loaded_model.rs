#![cfg(feature = "tests_that_use_llms")]

use paddler_agent::desired_state_reconciliation::DesiredStateReconciliation;
use paddler_messaging::agent_runtime_status::AgentRuntimeStatus;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_messaging::produces_snapshot::ProducesSnapshot as _;
use paddler_tests::kev_0_8b_desired_state::kev_0_8b_desired_state;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::serving_pipeline_arbiter::ServingPipelineArbiter;

use crate::swappable_model_file::SwappableModelFile;

#[tokio::test(flavor = "multi_thread")]
async fn enabling_decisions_reuses_the_loaded_model() {
    let swappable_model_file = SwappableModelFile::link_to(&kev_0_8b_target_path("model.gguf"));
    let decisions_desired_state = BalancerDesiredState {
        model: swappable_model_file.agent_desired_model(),
        ..kev_0_8b_desired_state()
    };
    let mut serving_pipeline_arbiter = ServingPipelineArbiter::new(DECISION_SLOTS_MINIMUM.get());

    assert_eq!(
        serving_pipeline_arbiter
            .serve(BalancerDesiredState {
                inference_mode: InferenceMode::TextGenerationAndEmbeddings,
                ..decisions_desired_state.clone()
            })
            .await,
        DesiredStateReconciliation::Reconciled
    );

    swappable_model_file.replace_with_an_unloadable_file();

    assert_eq!(
        serving_pipeline_arbiter
            .serve(decisions_desired_state)
            .await,
        DesiredStateReconciliation::Reconciled
    );
    assert_eq!(
        serving_pipeline_arbiter
            .inference_runtime_context
            .slot_aggregated_status
            .make_snapshot()
            .status
            .runtime,
        AgentRuntimeStatus::Serving {
            inference_mode: InferenceMode::TextGenerationEmbeddingsAndDecisions,
            slots_total: u64::from(DECISION_SLOTS_MINIMUM.get()),
        }
    );

    serving_pipeline_arbiter.shut_down().await;
}
