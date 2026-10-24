use std::process::Stdio;

use paddler_cli_tests::paddler_command::paddler_command;
use paddler_cli_tests::unreachable_management_addr::UNREACHABLE_MANAGEMENT_ADDR;
use paddler_test_cluster_harness::failed_run_exit_code::FAILED_RUN_EXIT_CODE;
use paddler_test_cluster_harness::openssl_config_activating_a_missing_provider::OPENSSL_CONFIG_ACTIVATING_A_MISSING_PROVIDER;

#[tokio::test(flavor = "multi_thread")]
async fn agent_exits_with_failure_when_it_cannot_set_up_its_model_downloader() {
    assert_eq!(
        paddler_command(env!("CARGO_BIN_EXE_paddler_cluster_node"))
            .arg("agent")
            .arg("--management-addr")
            .arg(UNREACHABLE_MANAGEMENT_ADDR.to_string())
            .arg("--slots")
            .arg("1")
            .env("OPENSSL_CONF", OPENSSL_CONFIG_ACTIVATING_A_MISSING_PROVIDER)
            .stderr(Stdio::null())
            .status()
            .await
            .expect("the agent must run")
            .code(),
        Some(FAILED_RUN_EXIT_CODE)
    );
}
