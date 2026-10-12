#![cfg(feature = "tests_that_use_llms")]

use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::token::LlamaToken;

use paddler_agent_decision::decision_capacity_ledger::DecisionCapacityLedger;
use paddler_agent_decision::decision_decoder::DecisionDecoder;
use paddler_messaging::decision_result::DecisionResult;
use paddler_tests::admitted_decision::AdmittedDecision;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;

const SEQUENCES_THE_CONTEXT_HOLDS: u32 = 1;
const SEQUENCES_THE_LEDGER_HANDS_OUT: u16 = 2;

#[test]
fn a_decision_whose_sequences_cannot_be_released_reports_the_release_failure() {
    let runtime = LlamaRuntime::load(&kev_0_8b_target_path("model.gguf"))
        .expect("the kev decision model must load");
    let mut llama_context = runtime
        .create_decision_llama_context(SEQUENCES_THE_CONTEXT_HOLDS)
        .expect("the decision context must be created");
    let n_batch = runtime.model_runtime_parameters.n_batch.tokens_usize();
    let mut batch = LlamaBatch::new(n_batch, 1).expect("the decision batch must be allocated");
    let AdmittedDecision {
        active_decision_request,
        mut decision_result_rx,
        decision_stop_tx: _decision_stop_tx,
    } = AdmittedDecision::admit(
        &DecisionCapacityLedger::new(
            llama_context.n_ctx() as usize,
            SEQUENCES_THE_LEDGER_HANDS_OUT,
        ),
        runtime
            .inference_runtime_context
            .slot_aggregated_status
            .clone(),
        vec![LlamaToken::new(0)],
    )
    .expect("the decision must be admitted");

    active_decision_request.finish_with(
        &mut DecisionDecoder {
            batch: &mut batch,
            llama_context: &mut llama_context,
            n_batch,
        },
        None,
        DecisionResult::StopRequested,
    );

    assert!(matches!(
        decision_result_rx.try_recv(),
        Ok(DecisionResult::KvCacheRemovalFailed(_))
    ));
}
