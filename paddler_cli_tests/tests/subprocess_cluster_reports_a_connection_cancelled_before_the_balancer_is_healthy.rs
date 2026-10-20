use std::mem::discriminant;

use tokio_util::sync::CancellationToken;

use paddler_cli_tests::subprocess_agent_spawner::SubprocessAgentSpawner;
use paddler_client::error::Error as ClientError;
use paddler_test_cluster_harness::cluster::Cluster;
use paddler_test_cluster_harness::cluster_desired_state::ClusterDesiredState;

use crate::running_balancer_subprocess::running_balancer_subprocess;

#[tokio::test(flavor = "multi_thread")]
async fn subprocess_cluster_reports_a_connection_cancelled_before_the_balancer_is_healthy() {
    let running_balancer = running_balancer_subprocess().await;
    let management_addr = running_balancer.addresses.management;
    let cancelled_connection = CancellationToken::new();

    cancelled_connection.cancel();

    assert_eq!(
        Cluster::connect(
            cancelled_connection,
            running_balancer,
            Box::new(SubprocessAgentSpawner::new(
                env!("CARGO_BIN_EXE_paddler_cluster_node").to_owned(),
                management_addr,
            )),
            &ClusterDesiredState::Apply(Box::default()),
        )
        .await
        .err()
        .as_ref()
        .and_then(|connection_error| connection_error.root_cause().downcast_ref::<ClientError>())
        .map(discriminant),
        Some(discriminant(&ClientError::RequestCancelled {
            url: String::new()
        }))
    );
}
