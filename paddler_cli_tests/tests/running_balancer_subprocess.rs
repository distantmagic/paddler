use paddler_cli_tests::signalled_subprocess::SignalledSubprocess;
use paddler_cli_tests::spawn_balancer_subprocess::spawn_balancer_subprocess;
use paddler_test_cluster_harness::ephemeral_loopback_addr::EPHEMERAL_LOOPBACK_ADDR;
use paddler_test_cluster_harness::running_balancer::RunningBalancer;

pub async fn running_balancer_subprocess() -> RunningBalancer {
    let ephemeral_loopback_addr = EPHEMERAL_LOOPBACK_ADDR.to_string();

    spawn_balancer_subprocess(
        env!("CARGO_BIN_EXE_paddler_cluster_node"),
        [
            "--inference-addr",
            &ephemeral_loopback_addr,
            "--management-addr",
            &ephemeral_loopback_addr,
        ],
        SignalledSubprocess::new,
    )
    .await
    .expect("the balancer must start and announce its addresses")
    .running_balancer
}
