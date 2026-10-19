#![cfg(feature = "tests_that_use_llms")]

use paddler_agent::desired_state_reconciliation::DesiredStateReconciliation;
use paddler_messaging::agent_runtime_status::AgentRuntimeStatus;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_messaging::produces_snapshot::ProducesSnapshot as _;
use paddler_tests::desired_state_with_chat_template_override::desired_state_with_chat_template_override;
use paddler_tests::kev_0_8b_desired_state::kev_0_8b_desired_state;
use paddler_tests::qwen3_desired_state::qwen3_desired_state;
use paddler_tests::serving_pipeline_arbiter::ServingPipelineArbiter;
use paddler_tests::uncompilable_chat_template::uncompilable_chat_template;

#[tokio::test(flavor = "multi_thread")]
async fn a_model_held_by_a_failed_start_is_released_before_another_model_loads() {
    let mut serving_pipeline_arbiter = ServingPipelineArbiter::new(1);

    assert!(matches!(
        serving_pipeline_arbiter
            .apply(desired_state_with_chat_template_override(
                BalancerDesiredState {
                    inference_mode: InferenceMode::TextGenerationAndEmbeddings,
                    ..kev_0_8b_desired_state()
                        .expect("the kev desired state must name absolute paths")
                },
                uncompilable_chat_template(),
            ))
            .await,
        DesiredStateReconciliation::Reconciled(_)
    ));

    serving_pipeline_arbiter.conclude_next_progress().await;
    serving_pipeline_arbiter.conclude_next_progress().await;

    assert!(matches!(
        serving_pipeline_arbiter.apply(qwen3_desired_state()).await,
        DesiredStateReconciliation::Reconciled(_)
    ));

    serving_pipeline_arbiter.conclude_next_progress().await;
    serving_pipeline_arbiter.conclude_next_progress().await;
    serving_pipeline_arbiter.conclude_next_progress().await;

    assert_eq!(
        serving_pipeline_arbiter
            .inference_runtime_context
            .slot_aggregated_status
            .make_snapshot()
            .status
            .runtime,
        AgentRuntimeStatus::Serving {
            inference_mode: InferenceMode::TextGenerationAndEmbeddings,
            slots_total: 1,
        }
    );

    serving_pipeline_arbiter.shut_down().await;
}
