use tokio_util::sync::CancellationToken;

use paddler_cli_tests::missing_binary_path::MISSING_BINARY_PATH;
use paddler_cli_tests::subprocess_agent_spawner::SubprocessAgentSpawner;
use paddler_test_cluster_harness::cluster::Cluster;
use paddler_test_cluster_harness::cluster_desired_state::ClusterDesiredState;

use crate::running_balancer_subprocess::running_balancer_subprocess;

pub async fn cluster_with_a_missing_agent_binary() -> Cluster {
    let running_balancer = running_balancer_subprocess().await;
    let management_addr = running_balancer.addresses.management;

    Cluster::connect(
        CancellationToken::new(),
        running_balancer,
        Box::new(SubprocessAgentSpawner::new(
            MISSING_BINARY_PATH.to_owned(),
            management_addr,
        )),
        &ClusterDesiredState::Apply(Box::default()),
    )
    .await
    .expect("the cluster must connect to the balancer")
}
