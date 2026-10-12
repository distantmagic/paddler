#![cfg(feature = "tests_that_use_llms")]

use std::collections::VecDeque;
use std::sync::mpsc::channel;

use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::token::LlamaToken;
use tokio_util::sync::CancellationToken;

use paddler_agent_decision::decision_capacity_ledger::DecisionCapacityLedger;
use paddler_agent_decision::decision_scheduler::DecisionScheduler;
use paddler_agent_decision::decision_scheduler_context::DecisionSchedulerContext;
use paddler_messaging::decision_result::DecisionResult;
use paddler_tests::admitted_decision::AdmittedDecision;
use paddler_tests::kev_0_8b_pointer_head::kev_0_8b_pointer_head;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;

#[test]
fn a_decision_running_when_the_agent_shuts_down_is_refused_as_unavailable() {
    let runtime = LlamaRuntime::load(&kev_0_8b_target_path("model.gguf"))
        .expect("the kev decision model must load");
    let llama_context = runtime
        .create_decision_llama_context(2)
        .expect("the decision context must be created");
    let n_batch = runtime.model_runtime_parameters.n_batch.tokens_usize();
    let capacity_ledger = DecisionCapacityLedger::new(llama_context.n_ctx() as usize, 2);
    let AdmittedDecision {
        active_decision_request,
        mut decision_result_rx,
        decision_stop_tx: _decision_stop_tx,
    } = AdmittedDecision::admit(
        &capacity_ledger,
        runtime
            .inference_runtime_context
            .slot_aggregated_status
            .clone(),
        vec![LlamaToken::new(0)],
    )
    .expect("the decision must be admitted");
    let agent_shutdown = CancellationToken::new();
    let (_scheduler_message_tx, scheduler_message_rx) = channel();

    agent_shutdown.cancel();

    DecisionScheduler {
        active_requests: VecDeque::from([active_decision_request]),
        agent_shutdown,
        batch: LlamaBatch::new(n_batch, 1).expect("the decision batch must be allocated"),
        capacity_ledger,
        llama_context,
        pending_requests: VecDeque::new(),
        scheduler_context: DecisionSchedulerContext {
            agent_name: None,
            n_batch,
            pointer_head: kev_0_8b_pointer_head().expect("the kev pointer head must load"),
        },
        scheduler_message_rx,
        shutdown_requested: false,
    }
    .run();

    assert!(matches!(
        decision_result_rx.try_recv(),
        Ok(DecisionResult::SchedulerUnavailable(_))
    ));
}
