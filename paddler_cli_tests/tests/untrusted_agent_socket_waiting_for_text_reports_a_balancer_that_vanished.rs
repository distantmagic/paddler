use std::mem::discriminant;

use tokio_tungstenite::tungstenite::Error as WebSocketError;

use paddler_test_cluster_harness::cluster_harness_error::ClusterHarnessError;

use crate::agent_socket_of_a_vanished_balancer::AgentSocketOfAVanishedBalancer;

#[tokio::test(flavor = "multi_thread")]
async fn untrusted_agent_socket_waiting_for_text_reports_a_balancer_that_vanished() {
    let mut agent_socket_of_a_vanished_balancer = AgentSocketOfAVanishedBalancer::connect().await;

    assert_eq!(
        agent_socket_of_a_vanished_balancer
            .agent_socket
            .next_text()
            .await
            .err()
            .as_ref()
            .map(discriminant),
        Some(discriminant(
            &ClusterHarnessError::AgentSocketReceiveFailed(WebSocketError::ConnectionClosed)
        ))
    );
    assert_eq!(
        agent_socket_of_a_vanished_balancer
            .agent_socket
            .next_text()
            .await
            .err()
            .as_ref()
            .map(discriminant),
        Some(discriminant(
            &ClusterHarnessError::AgentSocketEndedWithoutText
        ))
    );

    agent_socket_of_a_vanished_balancer.shut_down().await;
}
