#![cfg(feature = "tests_that_use_llms")]

use std::process::Command;

use paddler_test_cluster_harness::failed_run_exit_code::FAILED_RUN_EXIT_CODE;
use paddler_test_cluster_harness::openssl_config_activating_a_missing_provider::OPENSSL_CONFIG_ACTIVATING_A_MISSING_PROVIDER;

#[test]
fn run_with_test_cluster_fails_when_its_agents_cannot_set_up_their_model_downloader() {
    assert_eq!(
        Command::new(env!("CARGO_BIN_EXE_run_with_test_cluster"))
            .args(["qwen3-0-6b", "--", "true"])
            .env("OPENSSL_CONF", OPENSSL_CONFIG_ACTIVATING_A_MISSING_PROVIDER)
            .status()
            .expect("the test cluster runner must run")
            .code(),
        Some(FAILED_RUN_EXIT_CODE)
    );
}
