use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

use paddler_balancer::websocket_close_cause::WebSocketCloseCause;
use paddler_test_cluster_harness::cluster_harness_error::ClusterHarnessError;
use paddler_test_cluster_harness::cluster_params::ClusterParams;
use paddler_tests::start_cluster::start_cluster;

use crate::untrusted_management_agent::UntrustedManagementAgent;

#[tokio::test(flavor = "multi_thread")]
async fn an_untrusted_agent_waiting_for_a_request_reports_the_balancer_closing_its_socket() {
    let cluster = start_cluster(ClusterParams {
        agents: Vec::new(),
        wait_for_slots_ready: false,
        ..ClusterParams::default()
    })
    .await
    .expect("a cluster without agents must start");
    let mut closed_agent = UntrustedManagementAgent::connect(
        cluster.balancer.addresses.management,
        "closed-before-a-request",
    )
    .await
    .expect("the untrusted agent must reach the agent socket");

    closed_agent
        .socket_client
        .send(Message::text("not json"))
        .await
        .expect("the undeserializable message must be sent");

    let undeserializable_message_close_reason = WebSocketCloseCause::AgentMessageUndeserializable
        .close_reason()
        .expect("the balancer must give a reason for closing an undeserializable agent socket");

    assert_eq!(
        closed_agent
            .next_request()
            .await
            .err()
            .map(|closed_socket_error| closed_socket_error.to_string()),
        Some(
            ClusterHarnessError::AgentSocketClosedBeforeText {
                close_frame: Some(CloseFrame {
                    code: CloseCode::from(u16::from(undeserializable_message_close_reason.code)),
                    reason: undeserializable_message_close_reason
                        .description
                        .expect("the balancer must describe why it closes an undeserializable agent socket")
                        .into(),
                }),
            }
            .to_string()
        )
    );

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
