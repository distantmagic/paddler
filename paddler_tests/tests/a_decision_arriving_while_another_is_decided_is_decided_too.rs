#![cfg(feature = "tests_that_use_llms")]

use std::collections::VecDeque;
use std::mem::discriminant;
use std::sync::mpsc::channel;

use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::token::LlamaToken;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::sync::CancellationToken;

use paddler_agent_decision::decision_capacity_ledger::DecisionCapacityLedger;
use paddler_agent_decision::decision_delimiter_tokens::DecisionDelimiterTokens;
use paddler_agent_decision::decision_scheduler::DecisionScheduler;
use paddler_agent_decision::decision_scheduler_context::DecisionSchedulerContext;
use paddler_agent_decision::decision_token_layout::DecisionTokenLayout;
use paddler_agent_runtime::scheduler_inbox::SchedulerInbox;
use paddler_agent_runtime::scheduler_message::SchedulerMessage;
use paddler_messaging::decision_result::DecisionResult;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::decision_done_discriminant::decision_done_discriminant;
use paddler_tests::kev_0_8b_pointer_head::kev_0_8b_pointer_head;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;
use paddler_tests::prepared_decision::PreparedDecision;
use paddler_tests::question_answered_discriminant::question_answered_discriminant;
use paddler_tests::two_option_question::two_option_question;

const CONCURRENT_DECISIONS: u16 = 2;

#[test]
fn a_decision_arriving_while_another_is_decided_is_decided_too() {
    let runtime = LlamaRuntime::load(
        &kev_0_8b_target_path("model.gguf"),
        agent_inference_runtime_context(CONCURRENT_DECISIONS),
    )
    .expect("the kev decision model must load");
    let pointer_head = kev_0_8b_pointer_head().expect("the kev pointer head must load");
    let delimiter_tokens = DecisionDelimiterTokens::resolve(
        runtime.loaded_llama_model.model.as_ref(),
        &pointer_head.delimiters,
    )
    .expect("the kev delimiters must resolve");
    let prepared_decision = |question_id: &str| {
        PreparedDecision::new(
            runtime
                .inference_runtime_context
                .slot_aggregated_status
                .clone(),
            DecisionTokenLayout::new(
                &delimiter_tokens,
                vec![LlamaToken::new(0)],
                Vec::new(),
                two_option_question(question_id),
            ),
        )
    };
    let first_decision = prepared_decision("first");
    let second_decision = prepared_decision("second");
    let (scheduler_message_tx, scheduler_message_rx) = channel();

    for prepared_decision_request in [
        first_decision.prepared_decision_request,
        second_decision.prepared_decision_request,
    ] {
        scheduler_message_tx
            .send(SchedulerMessage::Command(prepared_decision_request))
            .expect("the scheduler must receive the decision");
    }

    drop(scheduler_message_tx);

    DecisionScheduler {
        active_requests: VecDeque::new(),
        agent_shutdown: CancellationToken::new(),
        batch: LlamaBatch::new(runtime.model_runtime_parameters.n_batch.capacity(), 1),
        capacity_ledger: DecisionCapacityLedger::new(CONCURRENT_DECISIONS),
        llama_context: runtime
            .create_decision_llama_context(u32::from(CONCURRENT_DECISIONS))
            .expect("the decision context must be created"),
        pending_requests: VecDeque::new(),
        scheduler_context: DecisionSchedulerContext {
            agent_name: None,
            n_batch: runtime.model_runtime_parameters.n_batch.tokens_usize(),
            pointer_head,
        },
        scheduler_inbox: SchedulerInbox::new(scheduler_message_rx),
    }
    .run();

    let assert_decided = |mut decision_result_rx: UnboundedReceiver<DecisionResult>| {
        assert_eq!(
            [decision_result_rx.try_recv(), decision_result_rx.try_recv()]
                .map(|decision_result| decision_result
                    .map(|decision_result| discriminant(&decision_result))),
            [
                Ok(question_answered_discriminant()),
                Ok(decision_done_discriminant())
            ]
        );
    };

    assert_decided(first_decision.decision_result_rx);
    assert_decided(second_decision.decision_result_rx);
}
