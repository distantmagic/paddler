#![cfg(feature = "tests_that_use_llms")]

use std::mem::discriminant;

use futures_util::StreamExt as _;
use tokio::spawn;
use tokio_util::sync::CancellationToken;

use paddler_test_cluster_harness::unending_generation::unending_generation;
use paddler_tests::decision_cluster_params::DecisionClusterParams;
use paddler_tests::decision_done_discriminant::decision_done_discriminant;
use paddler_tests::sample_decision::sample_decision;
use paddler_tests::start_decision_cluster::start_decision_cluster;

#[tokio::test(flavor = "multi_thread")]
async fn a_decision_with_leading_questions_waits_for_two_free_slots() {
    let mut cluster = start_decision_cluster(DecisionClusterParams::default())
        .await
        .expect("a decision cluster must start");
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

    let decision = spawn(cluster.decide(CancellationToken::new(), &sample_decision()));

    cluster
        .wait_for_buffered_request_count(1)
        .await
        .expect("the decision must wait while only one slot is free");
    generation_stop.cancel();

    assert!(
        unending_generation_stream.next().await.is_none(),
        "a cancelled generation must end its stream"
    );
    assert_eq!(
        discriminant(
            &decision
                .await
                .expect("the decision task must not panic")
                .expect("the decision must be answered")
                .terminal_result
        ),
        decision_done_discriminant()
    );

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
