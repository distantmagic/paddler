use reqwest::StatusCode;
use serde_json::json;

use paddler_balancer::compatibility::typesafe_service::typesafe_header::TypeSafeHeader;
use paddler_messaging::agent_desired_model::AgentDesiredModel;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_test_cluster_harness::cluster_desired_state::ClusterDesiredState;
use paddler_test_cluster_harness::cluster_params::ClusterParams;
use paddler_tests::cluster_without_agents_serving::cluster_without_agents_serving;
use paddler_tests::start_cluster::start_cluster;

#[tokio::test(flavor = "multi_thread")]
async fn typesafe_models_lists_nothing_while_no_agent_serves_the_desired_model() {
    let cluster = start_cluster(ClusterParams {
        desired_state: ClusterDesiredState::Apply(Box::new(BalancerDesiredState {
            model: AgentDesiredModel::Uri(
                "https://huggingface.co/intentee/kev/blob/main/kev-0.8b.gguf".to_owned(),
            ),
            inference_mode: InferenceMode::TextGenerationEmbeddingsAndDecisions,
            ..BalancerDesiredState::default()
        })),
        ..cluster_without_agents_serving(InferenceMode::TextGenerationEmbeddingsAndDecisions)
    })
    .await
    .expect("the cluster must start");
    let response = cluster
        .typesafe_models()
        .await
        .expect("the models request must be answered");

    assert_eq!(response.status, StatusCode::OK);
    assert!(
        response
            .headers
            .get(TypeSafeHeader::REQUEST_ID)
            .is_some_and(|request_id| !request_id.is_empty())
    );
    assert_eq!(response.body, json!({"models": []}));

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
