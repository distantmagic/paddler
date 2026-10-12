#![cfg(feature = "tests_that_use_llms")]

use std::sync::Arc;
use std::sync::mpsc::channel;

use hf_hub::Cache;
use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::mtmd::MtmdContext;
use llama_cpp_bindings::mtmd::MtmdContextParams;
use llama_cpp_bindings::mtmd::micro_batch_tokens;
use tokio_util::sync::CancellationToken;

use paddler_agent_runtime::scheduler_message::SchedulerMessage;
use paddler_agent_text_generation::continuous_batch_scheduler::ContinuousBatchScheduler;
use paddler_agent_text_generation::continuous_batch_scheduler_context::ContinuousBatchSchedulerContext;
use paddler_agent_text_generation::continuous_batch_scheduler_params::ContinuousBatchSchedulerParams;
use paddler_agent_text_generation::multimodal_prompt_support::MultimodalPromptSupport;
use paddler_agent_text_generation::prepared_generation_request::PreparedGenerationRequest;
use paddler_agent_text_generation::prepared_prompt::PreparedPrompt;
use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_test_cluster_harness::model_card::smolvlm2_256m::smolvlm2_256m;
use paddler_test_cluster_harness::model_card::smolvlm2_256m_mmproj::smolvlm2_256m_mmproj;
use paddler_tests::llama_runtime::LlamaRuntime;
use paddler_tests::prepared_generation::PreparedGeneration;

#[test]
fn a_multimodal_prompt_that_cannot_be_ingested_ends_the_generation() {
    let runtime = LlamaRuntime::load(
        &smolvlm2_256m()
            .reference
            .cached_path(&Cache::from_env())
            .expect("the vision model must be in the Hugging Face cache"),
    )
    .expect("the vision model must load");
    let llama_context = runtime
        .create_text_generation_llama_context(1)
        .expect("the context must be created");
    let sequence_context_size = llama_context.n_ctx_seq();
    let n_batch = runtime.model_runtime_parameters.n_batch;
    let mut ingestion = MultimodalPromptSupport {
        add_special_tokens: true,
        micro_batch_tokens: micro_batch_tokens(&llama_context, n_batch.tokens()),
        multimodal_context: Arc::new(
            MtmdContext::init_from_file(
                &smolvlm2_256m_mmproj()
                    .reference
                    .cached_path(&Cache::from_env())
                    .expect("the multimodal projection must be in the Hugging Face cache")
                    .to_string_lossy(),
                &runtime.loaded_llama_model.model,
                &MtmdContextParams::default(),
            )
            .expect("the multimodal projection must load"),
        ),
        n_batch: n_batch.tokens_i32(),
        sequence_context_size,
    }
    .prepare_prompt(&[], "Describe the picture".to_owned())
    .expect("the multimodal prompt must be prepared");

    ingestion.next_chunk_index = ingestion.chunks.len();

    let PreparedGeneration {
        generate_tokens_stop_tx: _generate_tokens_stop_tx,
        mut generated_tokens_rx,
        request,
    } = PreparedGeneration::of_raw_prompt(&runtime, sequence_context_size, "Describe the picture")
        .expect("the generation must be prepared");
    let (scheduler_message_tx, scheduler_message_rx) = channel();

    scheduler_message_tx
        .send(SchedulerMessage::Command(PreparedGenerationRequest {
            prompt: PreparedPrompt::Multimodal(ingestion),
            ..request
        }))
        .expect("the scheduler must accept the generation");
    scheduler_message_tx
        .send(SchedulerMessage::Shutdown)
        .expect("the scheduler must accept the shutdown");

    ContinuousBatchScheduler::new(ContinuousBatchSchedulerParams {
        agent_shutdown: CancellationToken::new(),
        batch: LlamaBatch::new(n_batch.tokens_usize(), 1).expect("the batch must be allocated"),
        llama_context,
        scheduler_context: ContinuousBatchSchedulerContext {
            agent_name: None,
            desired_slots_total: 1,
            model: runtime.loaded_llama_model.model.clone(),
            n_batch,
            sequence_context_size,
        },
        scheduler_message_rx,
    })
    .run();

    assert!(matches!(
        generated_tokens_rx.try_recv(),
        Ok(GeneratedTokenResult::MultimodalIngestionFailed(_))
    ));
}
