use std::num::NonZeroUsize;

use futures_util::StreamExt as _;
use tokio_util::sync::CancellationToken;

use paddler_client::inference_socket::reported_cluster_inference_mode::ReportedClusterInferenceMode;
use paddler_inference_parameters::embedding_parameters::EmbeddingParameters;
use paddler_messaging::balancer_desired_state::BalancerDesiredState;
use paddler_messaging::cluster_inference_mode_report::ClusterInferenceModeReport;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_tests::cluster_without_agents_serving::cluster_without_agents_serving;
use paddler_tests::desired_state_serving::desired_state_serving;
use paddler_tests::next_reported_cluster_inference_mode::next_reported_cluster_inference_mode;
use paddler_tests::rejected_conversation_stream::rejected_conversation_stream;
use paddler_tests::start_cluster::start_cluster;

#[tokio::test(flavor = "multi_thread")]
async fn inference_socket_notifies_clients_only_when_the_cluster_inference_mode_changes() {
    let cluster = start_cluster(cluster_without_agents_serving(
        InferenceMode::TextGenerationAndEmbeddings,
    ))
    .await
    .expect("a balancer serving text generation and embeddings must start");
    let mut cluster_inference_mode_rx = cluster
        .client_inference
        .subscribe_to_cluster_inference_mode();

    rejected_conversation_stream(&cluster.client_inference)
        .await
        .expect("the inference socket must accept the request")
        .next()
        .await
        .expect("the inference socket must answer the request")
        .expect("the answer must be readable");

    let connect_notification = next_reported_cluster_inference_mode(&mut cluster_inference_mode_rx)
        .await
        .expect("the client must be told on connect which inference mode the cluster serves");

    cluster
        .client_management
        .put_balancer_desired_state(
            CancellationToken::new(),
            &BalancerDesiredState {
                embeddings: EmbeddingParameters {
                    embedding_batch_size: NonZeroUsize::MIN,
                    ..EmbeddingParameters::default()
                },
                ..desired_state_serving(InferenceMode::TextGenerationAndEmbeddings)
            },
        )
        .await
        .expect("the balancer must apply a state that keeps its inference mode");

    let mut notification_after = async |inference_mode: InferenceMode| {
        cluster
            .client_management
            .put_balancer_desired_state(
                CancellationToken::new(),
                &desired_state_serving(inference_mode),
            )
            .await
            .expect("the balancer must apply the desired state");

        next_reported_cluster_inference_mode(&mut cluster_inference_mode_rx)
            .await
            .expect("the client must receive the inference mode change")
    };

    let decisions_notification =
        notification_after(InferenceMode::TextGenerationEmbeddingsAndDecisions).await;
    let without_decisions_notification =
        notification_after(InferenceMode::TextGenerationAndEmbeddings).await;

    assert!(matches!(
        [
            connect_notification,
            decisions_notification,
            without_decisions_notification,
        ],
        [
            ReportedClusterInferenceMode::Reported(ClusterInferenceModeReport {
                inference_mode: InferenceMode::TextGenerationAndEmbeddings,
                ..
            }),
            ReportedClusterInferenceMode::Reported(ClusterInferenceModeReport {
                inference_mode: InferenceMode::TextGenerationEmbeddingsAndDecisions,
                ..
            }),
            ReportedClusterInferenceMode::Reported(ClusterInferenceModeReport {
                inference_mode: InferenceMode::TextGenerationAndEmbeddings,
                ..
            }),
        ]
    ));

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
