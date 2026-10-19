use std::net::SocketAddr;

use tokio_util::sync::CancellationToken;

use paddler_bootstrap::agent_bootstrap_config::AgentBootstrapConfig;
use paddler_bootstrap::agent_runner::AgentRunner;
use paddler_bootstrap::agent_runner_params::AgentRunnerParams;
use paddler_cache_dir::cache_dir::CacheDir;
use paddler_test_cluster_harness::unexpiring_shutdown_options::unexpiring_shutdown_options;

#[must_use]
pub fn agent_runner_connected_to(management_addr: SocketAddr, slots: u16) -> AgentRunner {
    AgentRunner::start(AgentRunnerParams {
        bootstrap_config: AgentBootstrapConfig {
            agent_name: Some("test-agent".to_owned()),
            cache_dir: CacheDir::from_process_env(),
            management_address: management_addr.to_string(),
            slots,
        },
        cancellation_token: CancellationToken::new(),
        shutdown_options: unexpiring_shutdown_options(),
    })
}
