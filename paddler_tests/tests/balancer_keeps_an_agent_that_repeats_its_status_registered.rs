use tokio_tungstenite::tungstenite::Message;

use paddler_messaging::agent_status::AgentStatus;
use paddler_messaging::management_socket::balancer::notification::Notification as ManagementJsonRpcNotification;
use paddler_messaging::management_socket::balancer::notification_params::register_agent_params::RegisterAgentParams;
use paddler_messaging::management_socket::balancer::notification_params::update_agent_status_params::UpdateAgentStatusParams;
use paddler_messaging::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;
use paddler_test_cluster_harness::cluster_params::ClusterParams;
use paddler_test_cluster_harness::management_notification_text::management_notification_text;
use paddler_test_cluster_harness::untrusted_agent_socket_client::UntrustedAgentSocketClient;
use paddler_tests::start_cluster::start_cluster;

const REPEATING_AGENT_ID: &str = "repeating-agent";

#[tokio::test(flavor = "multi_thread")]
async fn balancer_keeps_an_agent_that_repeats_its_status_registered() {
    let mut cluster = start_cluster(ClusterParams {
        agents: Vec::new(),
        wait_for_slots_ready: false,
        ..ClusterParams::default()
    })
    .await
    .expect("a balancer without agents must start");
    let status_report = |desired_slots_total: u16, version: u64| {
        Message::text(
            management_notification_text(ManagementJsonRpcNotification::UpdateAgentStatus(
                UpdateAgentStatusParams {
                    slot_aggregated_status_snapshot: SlotAggregatedStatusSnapshot {
                        status: AgentStatus {
                            desired_slots_total,
                            ..AgentStatus::default()
                        },
                        version,
                    },
                },
            ))
            .expect("the status report must serialize"),
        )
    };
    let mut repeating_agent_socket = UntrustedAgentSocketClient::connect(
        cluster.balancer.addresses.management,
        REPEATING_AGENT_ID,
    )
    .await
    .expect("the repeating agent must reach the agent socket");

    repeating_agent_socket
        .send(Message::text(
            management_notification_text(ManagementJsonRpcNotification::RegisterAgent(
                RegisterAgentParams {
                    name: Some(REPEATING_AGENT_ID.to_owned()),
                    slot_aggregated_status_snapshot: SlotAggregatedStatusSnapshot {
                        status: AgentStatus {
                            desired_slots_total: 1,
                            ..AgentStatus::default()
                        },
                        version: 1,
                    },
                },
            ))
            .expect("the registration must serialize"),
        ))
        .await
        .expect("the registration must be sent");
    repeating_agent_socket
        .send(status_report(1, 2))
        .await
        .expect("the repeated status must be sent");
    repeating_agent_socket
        .send(status_report(2, 3))
        .await
        .expect("the changed status must be sent");

    cluster
        .agents_watcher
        .until(|snapshot| {
            snapshot.agents.iter().any(|agent| {
                agent.id == REPEATING_AGENT_ID && agent.status.desired_slots_total == 2
            })
        })
        .await
        .expect("the balancer must keep the agent and take its changed status");

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
