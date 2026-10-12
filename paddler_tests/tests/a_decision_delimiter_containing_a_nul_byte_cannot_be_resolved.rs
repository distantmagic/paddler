#![cfg(feature = "tests_that_use_llms")]

use llama_cpp_bindings::error::StringToTokenError;

use paddler_agent_decision::decision_delimiter_tokens::DecisionDelimiterTokens;
use paddler_agent_decision::decision_error::DecisionError;
use paddler_agent_pointer_head::pointer_head_delimiters::PointerHeadDelimiters;
use paddler_tests::kev_0_8b_pointer_head::kev_0_8b_pointer_head;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;

#[test]
fn a_decision_delimiter_containing_a_nul_byte_cannot_be_resolved() {
    let runtime = LlamaRuntime::load(&kev_0_8b_target_path("model.gguf"))
        .expect("the kev decision model must load");
    let pointer_head = kev_0_8b_pointer_head().expect("the kev pointer head must load");
    let delimiter_with_nul_byte = format!("{}\0", pointer_head.delimiters.decide);

    assert!(matches!(
        DecisionDelimiterTokens::resolve(
            runtime.loaded_llama_model.model.as_ref(),
            &PointerHeadDelimiters {
                decide: delimiter_with_nul_byte.clone(),
                ..pointer_head.delimiters
            },
        ),
        Err(DecisionError::DelimiterTokenizationFailed {
            delimiter,
            source: StringToTokenError::NulError(_),
        }) if delimiter == delimiter_with_nul_byte
    ));
}
