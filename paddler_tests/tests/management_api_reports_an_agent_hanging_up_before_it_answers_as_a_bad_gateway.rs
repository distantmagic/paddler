use reqwest::Client;
use reqwest::StatusCode;
use serde_json::to_string;
use tokio::spawn;
use tokio_tungstenite::tungstenite::Message;

use paddler_messaging::agent_status::AgentStatus;
use paddler_messaging::api_path::ApiPath;
use paddler_messaging::management_socket::balancer::message::Message as ManagementJsonRpcMessage;
use paddler_messaging::management_socket::balancer::notification::Notification as ManagementJsonRpcNotification;
use paddler_messaging::management_socket::balancer::notification_params::register_agent_params::RegisterAgentParams;
use paddler_messaging::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;
use paddler_test_cluster_harness::cluster_params::ClusterParams;
use paddler_test_cluster_harness::untrusted_agent_socket_client::UntrustedAgentSocketClient;
use paddler_tests::start_cluster::start_cluster;

const AGENT_ID: &str = "hanging-up";

#[tokio::test(flavor = "multi_thread")]
async fn management_api_reports_an_agent_hanging_up_before_it_answers_as_a_bad_gateway() {
    let mut cluster = start_cluster(ClusterParams {
        agents: Vec::new(),
        wait_for_slots_ready: false,
        ..ClusterParams::default()
    })
    .await
    .expect("a cluster without agents must start");
    let mut hanging_up_agent =
        UntrustedAgentSocketClient::connect(cluster.balancer.addresses.management, AGENT_ID)
            .await
            .expect("the agent must reach the agent socket");

    hanging_up_agent
        .send(Message::text(
            to_string(&ManagementJsonRpcMessage::Notification(
                ManagementJsonRpcNotification::RegisterAgent(RegisterAgentParams {
                    name: Some(AGENT_ID.to_owned()),
                    slot_aggregated_status_snapshot: SlotAggregatedStatusSnapshot {
                        status: AgentStatus::default(),
                        version: 1,
                    },
                }),
            ))
            .expect("the registration must serialize"),
        ))
        .await
        .expect("the registration must be sent");
    cluster
        .wait_for_agent_count(1)
        .await
        .expect("the balancer must register the agent");

    let model_metadata_response = spawn(
        Client::new()
            .get(
                cluster
                    .balancer
                    .management_base_url()
                    .expect("the balancer must serve the management API")
                    .join(&ApiPath::agent_model_metadata(AGENT_ID))
                    .expect("the model metadata path must form a URL"),
            )
            .send(),
    );

    hanging_up_agent
        .next_request()
        .await
        .expect("the balancer must ask the agent for its model metadata");
    drop(hanging_up_agent);

    assert_eq!(
        model_metadata_response
            .await
            .expect("the model metadata request must finish")
            .expect("the balancer must answer the model metadata request")
            .status(),
        StatusCode::BAD_GATEWAY
    );

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
}
