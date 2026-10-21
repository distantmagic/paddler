use futures_util::StreamExt as _;

use paddler_client::inference_socket::reported_cluster_inference_mode::ReportedClusterInferenceMode;
use paddler_messaging::cluster_inference_mode_report::ClusterInferenceModeReport;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_tests::cluster_without_agents_serving::cluster_without_agents_serving;
use paddler_tests::rejected_conversation_stream::rejected_conversation_stream;
use paddler_tests::start_cluster::start_cluster;

#[tokio::test(flavor = "multi_thread")]
async fn a_late_cluster_inference_mode_subscriber_sees_the_current_mode() {
    let cluster = start_cluster(cluster_without_agents_serving(
        InferenceMode::TextGenerationAndEmbeddings,
    ))
    .await
    .expect("a balancer serving text generation and embeddings must start");

    rejected_conversation_stream(&cluster.client_inference)
        .await
        .expect("the inference socket must accept the request before anyone subscribes")
        .next()
        .await
        .expect("the inference socket must answer before anyone subscribes")
        .expect("the answer must be readable");

    assert!(matches!(
        *cluster
            .client_inference
            .subscribe_to_cluster_inference_mode()
            .borrow(),
        ReportedClusterInferenceMode::Reported(ClusterInferenceModeReport {
            inference_mode: InferenceMode::TextGenerationAndEmbeddings,
            ..
        })
    ));

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
