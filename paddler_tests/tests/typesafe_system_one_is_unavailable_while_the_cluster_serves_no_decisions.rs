use reqwest::StatusCode;
use serde_json::json;

use paddler_messaging::inference_mode::InferenceMode;
use paddler_test_cluster_harness::noul_system_one_request::noul_system_one_request;
use paddler_tests::cluster_without_agents_serving::cluster_without_agents_serving;
use paddler_tests::start_cluster::start_cluster;

#[tokio::test(flavor = "multi_thread")]
async fn typesafe_system_one_is_unavailable_while_the_cluster_serves_no_decisions() {
    let cluster = start_cluster(cluster_without_agents_serving(
        InferenceMode::TextGenerationAndEmbeddings,
    ))
    .await
    .expect("a balancer serving text generation and embeddings must start");

    let response = cluster
        .typesafe_system_one(
            "text-generation-cluster-request",
            &noul_system_one_request(),
        )
        .await
        .expect("the System One request must be answered");

    assert_eq!(response.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        response.body,
        json!({"detail": "The cluster serves TextGenerationAndEmbeddings, which does not include Decisions"})
    );

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
