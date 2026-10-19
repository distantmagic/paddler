#![cfg(feature = "tests_that_use_llms")]

use futures_util::StreamExt as _;
use tokio_util::sync::CancellationToken;

use paddler_messaging::embedding_input_document::EmbeddingInputDocument;
use paddler_messaging::embedding_normalization_method::EmbeddingNormalizationMethod;
use paddler_messaging::request_params::generate_embedding_batch_params::GenerateEmbeddingBatchParams;
use paddler_test_cluster_harness::agent_config::AgentConfig;
use paddler_test_cluster_harness::cluster_desired_state::ClusterDesiredState;
use paddler_test_cluster_harness::cluster_params::ClusterParams;
use paddler_test_cluster_harness::unending_generation::unending_generation;
use paddler_tests::qwen3_desired_state::qwen3_desired_state;
use paddler_tests::start_cluster::start_cluster;

const SLOT_COUNT: u16 = 2;

#[tokio::test(flavor = "multi_thread")]
async fn an_embedding_batch_runs_on_the_slots_its_agent_has_free() {
    let mut cluster = start_cluster(ClusterParams {
        agents: vec![AgentConfig::single(SLOT_COUNT)],
        desired_state: ClusterDesiredState::Apply(Box::new(qwen3_desired_state())),
        wait_for_slots_ready: true,
        ..ClusterParams::default()
    })
    .await
    .expect("the cluster must start");
    let agent_id = cluster
        .agent_ids
        .first()
        .expect("the cluster must register its agent")
        .clone();
    let generation_stop = CancellationToken::new();
    let mut unending_generation_stream = cluster
        .client_inference
        .continue_from_raw_prompt(generation_stop.clone(), unending_generation())
        .await
        .expect("the generation must be accepted");

    unending_generation_stream
        .next()
        .await
        .expect("the generation must stream a message while it holds its slot")
        .expect("the message must be readable");
    cluster
        .wait_for_slots_processing(&agent_id, 1)
        .await
        .expect("the generation must hold one slot");

    let embedded = cluster
        .generate_embedding_batch(
            CancellationToken::new(),
            &GenerateEmbeddingBatchParams {
                input_batch: (0..SLOT_COUNT)
                    .map(|document_index| EmbeddingInputDocument {
                        content: format!("Document number {document_index}."),
                        id: format!("doc-{document_index}"),
                    })
                    .collect(),
                normalization_method: EmbeddingNormalizationMethod::None,
            },
        )
        .await
        .expect("the embedding batch must be accepted");

    assert_eq!(
        embedded
            .embeddings
            .iter()
            .map(|embedding_with_producer| embedding_with_producer
                .embedding
                .source_document_id
                .clone())
            .collect::<Vec<_>>(),
        vec!["doc-0".to_owned(), "doc-1".to_owned()]
    );

    generation_stop.cancel();

    assert!(
        unending_generation_stream.next().await.is_none(),
        "a cancelled generation must end its stream"
    );

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
