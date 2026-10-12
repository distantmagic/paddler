use paddler_cli_tests::pausable_agent::PausableAgent;
use paddler_cli_tests::pausable_agent_params::PausableAgentParams;
use paddler_cli_tests::stallable_tcp_relay::StallableTcpRelay;
use paddler_cli_tests::start_subprocess_cluster::start_subprocess_cluster;
use paddler_test_cluster_harness::agent_config::AgentConfig;
use paddler_test_cluster_harness::cluster_params::ClusterParams;

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_reconnects_when_the_balancer_stops_answering_pings() {
    let mut cluster = start_subprocess_cluster(
        env!("CARGO_BIN_EXE_paddler_cluster_node"),
        ClusterParams {
            agents: Vec::new(),
            wait_for_slots_ready: false,
            ..ClusterParams::default()
        },
    )
    .await
    .expect("a cluster without agents must start");
    let relay = StallableTcpRelay::start(cluster.balancer.addresses.management)
        .await
        .expect("the relay to the balancer must start");

    PausableAgent::join(
        &mut cluster,
        PausableAgentParams {
            binary_path: env!("CARGO_BIN_EXE_paddler_cluster_node").to_owned(),
            config: AgentConfig::single(1),
            expected_slots_total: 0,
            management_addr: relay.addr,
        },
    )
    .await
    .expect("the agent must register through the relay");

    relay.stall_established_connections();
    relay
        .wait_for_accepted_connections(2)
        .await
        .expect("the agent must reconnect once the balancer stops answering its pings");

    cluster
        .shutdown()
        .await
        .expect("the cluster must shut down cleanly");
    relay
        .stop()
        .await
        .expect("the relay must stop without a relaying failure");
}
