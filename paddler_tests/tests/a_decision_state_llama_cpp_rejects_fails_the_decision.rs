#![cfg(feature = "tests_that_use_llms")]

use llama_cpp_bindings::error::DecodeError;
use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::token::LlamaToken;

use paddler_agent_decision::decision_capacity_ledger::DecisionCapacityLedger;
use paddler_agent_decision::decision_decoder::DecisionDecoder;
use paddler_agent_decision::decision_error::DecisionError;
use paddler_tests::admitted_decision::AdmittedDecision;
use paddler_tests::kev_0_8b_pointer_head::kev_0_8b_pointer_head;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;

#[test]
fn a_decision_state_llama_cpp_rejects_fails_the_decision() {
    let runtime = LlamaRuntime::load(&kev_0_8b_target_path("model.gguf"))
        .expect("the kev decision model must load");
    let pointer_head = kev_0_8b_pointer_head().expect("the kev pointer head must load");
    let mut llama_context = runtime
        .create_decision_llama_context(2)
        .expect("the decision context must be created");
    let n_batch = runtime.model_runtime_parameters.n_batch.tokens_usize();
    let mut batch = LlamaBatch::new(n_batch, 1).expect("the decision batch must be allocated");
    let mut admitted_decision = AdmittedDecision::admit(
        &DecisionCapacityLedger::new(llama_context.n_ctx() as usize, 2),
        runtime
            .inference_runtime_context
            .slot_aggregated_status
            .clone(),
        vec![LlamaToken::new(runtime.loaded_llama_model.model.n_vocab())],
    )
    .expect("the decision must be admitted");

    assert_eq!(
        admitted_decision
            .active_decision_request
            .advance(
                &mut DecisionDecoder {
                    batch: &mut batch,
                    llama_context: &mut llama_context,
                    n_batch,
                },
                &pointer_head,
            )
            .err()
            .as_ref()
            .map(ToString::to_string),
        Some(DecisionError::DecodeFailed(DecodeError::BatchInvalid).to_string())
    );
}
