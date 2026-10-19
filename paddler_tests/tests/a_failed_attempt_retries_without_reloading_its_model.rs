#![cfg(feature = "tests_that_use_llms")]

use std::collections::BTreeSet;

use paddler_agent::desired_state_reconciliation::DesiredStateReconciliation;
use paddler_messaging::agent_issue::AgentIssue;
use paddler_messaging::agent_issue_params::chat_template_does_not_compile_params::ChatTemplateDoesNotCompileParams;
use paddler_messaging::agent_issue_params::model_path::ModelPath;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::chat_template::ChatTemplate;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_messaging::produces_snapshot::ProducesSnapshot as _;
use paddler_tests::desired_state_with_chat_template_override::desired_state_with_chat_template_override;
use paddler_tests::kev_0_8b_desired_state::kev_0_8b_desired_state;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::serving_pipeline_arbiter::ServingPipelineArbiter;

use crate::swappable_model_file::SwappableModelFile;

const UNCOMPILABLE_CHAT_TEMPLATE: &str = "{{invalid jinja template";

#[tokio::test(flavor = "multi_thread")]
async fn a_failed_attempt_retries_without_reloading_its_model() {
    let swappable_model_file = SwappableModelFile::link_to(&kev_0_8b_target_path("model.gguf"));
    let mut serving_pipeline_arbiter = ServingPipelineArbiter::new(1);

    assert_eq!(
        serving_pipeline_arbiter
            .apply(desired_state_with_chat_template_override(
                BalancerDesiredState {
                    inference_mode: InferenceMode::TextGenerationAndEmbeddings,
                    model: swappable_model_file.agent_desired_model(),
                    ..kev_0_8b_desired_state()
                },
                ChatTemplate {
                    content: UNCOMPILABLE_CHAT_TEMPLATE.to_owned(),
                },
            ))
            .await,
        DesiredStateReconciliation::Reconciled
    );

    let failed_attempt = serving_pipeline_arbiter
        .pipeline_arbiter
        .next_finished_startup()
        .await;

    swappable_model_file.replace_with_an_unloadable_file();
    serving_pipeline_arbiter
        .pipeline_arbiter
        .conclude(failed_attempt);

    let retried_attempt = serving_pipeline_arbiter
        .pipeline_arbiter
        .next_finished_startup()
        .await;

    serving_pipeline_arbiter
        .pipeline_arbiter
        .conclude(retried_attempt);

    assert_eq!(
        serving_pipeline_arbiter
            .inference_runtime_context
            .slot_aggregated_status
            .make_snapshot()
            .status
            .issues,
        BTreeSet::from([AgentIssue::ChatTemplateDoesNotCompile(
            ChatTemplateDoesNotCompileParams {
                error: "syntax error: unexpected identifier, expected end of variable block (in chat_template:1)".to_owned(),
                model_path: ModelPath::from(swappable_model_file.model_path().as_path()),
                template_content: UNCOMPILABLE_CHAT_TEMPLATE.to_owned(),
            }
        )])
    );

    serving_pipeline_arbiter.shut_down().await;
}
