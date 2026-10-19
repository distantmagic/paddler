#![cfg(feature = "tests_that_use_llms")]

use std::mem::discriminant;
use std::path::Path;

use tokio::spawn;
use tokio_util::sync::CancellationToken;

use paddler_messaging::agent_issue::AgentIssue;
use paddler_messaging::agent_issue_params::model_path::ModelPath;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::decision_settings::DecisionSettings;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_test_cluster_harness::agent_desired_model_at::agent_desired_model_at;
use paddler_tests::decision_cluster_params::DecisionClusterParams;
use paddler_tests::decision_done_discriminant::decision_done_discriminant;
use paddler_tests::sample_decision::sample_decision;
use paddler_tests::start_decision_cluster::start_decision_cluster;

const MISSING_POINTER_HEAD_PATH: &str = "/nonexistent/pointer_head.gguf";

#[tokio::test(flavor = "multi_thread")]
async fn a_request_waits_while_agents_still_serve_the_previous_inference_mode() {
    let mut cluster = start_decision_cluster(DecisionClusterParams::default())
        .await
        .expect("a decision cluster must start");

    let decisions_desired_state = cluster
        .client_management
        .get_balancer_desired_state(CancellationToken::new())
        .await
        .expect("the balancer must report its desired state");

    cluster
        .client_management
        .put_balancer_desired_state(
            CancellationToken::new(),
            &BalancerDesiredState {
                inference_mode: InferenceMode::TextGenerationAndEmbeddings,
                ..decisions_desired_state.clone()
            },
        )
        .await
        .expect("the balancer must stop serving decisions");
    cluster
        .wait_for_first_agent_to_serve(InferenceMode::TextGenerationAndEmbeddings)
        .await
        .expect("the agent must stop serving decisions");
    cluster
        .client_management
        .put_balancer_desired_state(
            CancellationToken::new(),
            &BalancerDesiredState {
                decision: DecisionSettings {
                    pointer_head: agent_desired_model_at(Path::new(MISSING_POINTER_HEAD_PATH))
                        .expect("the missing pointer head path must be absolute"),
                },
                ..decisions_desired_state.clone()
            },
        )
        .await
        .expect("the balancer must serve decisions again");
    cluster
        .wait_for_first_agent_issue(|issue| {
            *issue
                == AgentIssue::PointerHeadCannotBeLoaded(ModelPath {
                    model_path: MISSING_POINTER_HEAD_PATH.to_owned(),
                })
        })
        .await
        .expect("the agent must report the pointer head it cannot load");

    let decision = spawn(cluster.decide(CancellationToken::new(), &sample_decision()));

    cluster
        .wait_for_buffered_request_count(1)
        .await
        .expect("the decision must wait in the buffer");

    cluster
        .client_management
        .put_balancer_desired_state(CancellationToken::new(), &decisions_desired_state)
        .await
        .expect("the balancer must accept a pointer head the agent can load");

    assert_eq!(
        discriminant(
            &decision
                .await
                .expect("the decision task must not panic")
                .expect("the buffered decision must be answered")
                .terminal_result
        ),
        decision_done_discriminant()
    );

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
