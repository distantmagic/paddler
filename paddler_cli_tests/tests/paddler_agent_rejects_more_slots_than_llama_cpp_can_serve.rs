use std::process::Command;

use paddler_bootstrap::llama_cpp_max_sequences::LLAMA_CPP_MAX_SEQUENCES;
use paddler_cli_tests::clap_usage_error_exit_code::CLAP_USAGE_ERROR_EXIT_CODE;

#[test]
fn paddler_agent_rejects_more_slots_than_llama_cpp_can_serve() {
    let agent_output = Command::new(env!("CARGO_BIN_EXE_paddler_cluster_node"))
        .arg("agent")
        .arg("--management-addr")
        .arg("127.0.0.1:1")
        .arg("--slots")
        .arg((LLAMA_CPP_MAX_SEQUENCES + 1).to_string())
        .output()
        .expect("paddler agent must run");

    assert_eq!(agent_output.status.code(), Some(CLAP_USAGE_ERROR_EXIT_CODE));
    assert_eq!(agent_output.stdout, Vec::<u8>::new());
}
