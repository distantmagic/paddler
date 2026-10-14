#![cfg(feature = "tests_that_use_llms")]

use std::sync::mpsc::channel;

use llama_cpp_bindings::batch_add_error::BatchAddError;
use llama_cpp_bindings::llama_batch::LlamaBatch;
use tokio_util::sync::CancellationToken;

use paddler_agent_runtime::scheduler_message::SchedulerMessage;
use paddler_agent_text_generation::continuous_batch_scheduler::ContinuousBatchScheduler;
use paddler_agent_text_generation::continuous_batch_scheduler_context::ContinuousBatchSchedulerContext;
use paddler_agent_text_generation::continuous_batch_scheduler_params::ContinuousBatchSchedulerParams;
use paddler_agent_text_generation::text_generation_error::TextGenerationError;
use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_tests::kev_0_8b_target_path::kev_0_8b_target_path;
use paddler_tests::llama_runtime::LlamaRuntime;
use paddler_tests::prepared_generation::PreparedGeneration;

const TOKENS_THE_BATCH_HOLDS: usize = 1;

#[test]
fn a_generation_overflowing_its_batch_fails_as_batch_assembly() {
    let runtime =
        LlamaRuntime::load(&kev_0_8b_target_path("model.gguf")).expect("the kev model must load");
    let llama_context = runtime
        .create_text_generation_llama_context(1)
        .expect("the context must be created");
    let sequence_context_size = llama_context.n_ctx_seq();
    let PreparedGeneration {
        generate_tokens_stop_tx: _generate_tokens_stop_tx,
        mut generated_tokens_rx,
        request,
    } = PreparedGeneration::of_raw_prompt(
        &runtime,
        sequence_context_size,
        "Paddler balances requests across agents",
    )
    .expect("the generation must be prepared");
    let (scheduler_message_tx, scheduler_message_rx) = channel();

    scheduler_message_tx
        .send(SchedulerMessage::Command(request))
        .expect("the scheduler must accept the generation");
    scheduler_message_tx
        .send(SchedulerMessage::Shutdown)
        .expect("the scheduler must accept the shutdown");

    ContinuousBatchScheduler::new(ContinuousBatchSchedulerParams {
        agent_shutdown: CancellationToken::new(),
        batch: LlamaBatch::new(TOKENS_THE_BATCH_HOLDS, 1).expect("the batch must be allocated"),
        llama_context,
        scheduler_context: ContinuousBatchSchedulerContext {
            agent_name: None,
            desired_slots_total: 1,
            model: runtime.loaded_llama_model.model.clone(),
            n_batch: runtime.model_runtime_parameters.n_batch,
            sequence_context_size,
        },
        scheduler_message_rx,
    })
    .run();

    assert_eq!(
        generated_tokens_rx.try_recv(),
        Ok(GeneratedTokenResult::BatchAssemblyFailed(
            TextGenerationError::BatchAssemblyFailed(BatchAddError::InsufficientSpace(
                TOKENS_THE_BATCH_HOLDS
            ))
            .to_string()
        ))
    );
}
