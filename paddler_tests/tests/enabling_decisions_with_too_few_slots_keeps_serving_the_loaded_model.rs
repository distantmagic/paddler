#![cfg(feature = "tests_that_use_llms")]

use std::collections::BTreeSet;

use paddler_agent::desired_state_reconciliation::DesiredStateReconciliation;
use paddler_messaging::agent_issue::AgentIssue;
use paddler_messaging::agent_issue_params::slots_insufficient_for_decisions_params::SlotsInsufficientForDecisionsParams;
use paddler_messaging::agent_runtime_status::AgentRuntimeStatus;
use paddler_messaging::agent_state_application_status::AgentStateApplicationStatus;
use paddler_messaging::agent_status::AgentStatus;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_messaging::model_download_status::ModelDownloadStatus;
use paddler_messaging::produces_snapshot::ProducesSnapshot as _;
use paddler_tests::kev_0_8b_desired_state::kev_0_8b_desired_state;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::serving_pipeline_arbiter::ServingPipelineArbiter;

const SLOTS_TOO_FEW_FOR_DECISIONS: u16 = 1;

#[tokio::test(flavor = "multi_thread")]
async fn enabling_decisions_with_too_few_slots_keeps_serving_the_loaded_model() {
    let mut serving_pipeline_arbiter = ServingPipelineArbiter::new(SLOTS_TOO_FEW_FOR_DECISIONS);

    serving_pipeline_arbiter
        .serve(BalancerDesiredState {
            inference_mode: InferenceMode::TextGenerationAndEmbeddings,
            ..kev_0_8b_desired_state().expect("the kev desired state must name absolute paths")
        })
        .await;
    assert!(matches!(
        serving_pipeline_arbiter
            .apply(
                kev_0_8b_desired_state().expect("the kev desired state must name absolute paths")
            )
            .await,
        DesiredStateReconciliation::Reconciled(_)
    ));
    serving_pipeline_arbiter.conclude_next_progress().await;
    assert_eq!(
        serving_pipeline_arbiter
            .inference_runtime_context
            .slot_aggregated_status
            .make_snapshot()
            .status,
        AgentStatus {
            desired_slots_total: SLOTS_TOO_FEW_FOR_DECISIONS,
            download_status: ModelDownloadStatus::NotDownloading,
            issues: BTreeSet::from([AgentIssue::SlotsInsufficientForDecisions(
                SlotsInsufficientForDecisionsParams {
                    desired_slots: SLOTS_TOO_FEW_FOR_DECISIONS,
                    required_slots: DECISION_SLOTS_MINIMUM.get(),
                }
            )]),
            model_path: Some(kev_0_8b_target_path("model.gguf").display().to_string()),
            runtime: AgentRuntimeStatus::Serving {
                inference_mode: InferenceMode::TextGenerationAndEmbeddings,
                slots_total: u64::from(SLOTS_TOO_FEW_FOR_DECISIONS),
            },
            state_application_status: AgentStateApplicationStatus::AttemptedAndRetrying,
            uses_chat_template_override: false,
        }
    );

    serving_pipeline_arbiter.shut_down().await;
}
