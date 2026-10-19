#![cfg(feature = "tests_that_use_llms")]

use std::num::NonZeroU16;

use hf_hub::Cache;
use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::token::LlamaToken;
use tokio::sync::mpsc::error::TryRecvError;

use paddler_agent_embeddings::embedding_batch_processor::EmbeddingBatchProcessor;
use paddler_agent_embeddings::embedding_input_tokenized::EmbeddingInputTokenized;
use paddler_agent_embeddings::embedding_normalization::EmbeddingNormalization;
use paddler_agent_embeddings::embedding_scheduler_context::EmbeddingSchedulerContext;
use paddler_inference_parameters::pooling_type::PoolingType;
use paddler_messaging::embedding_result::EmbeddingResult;
use paddler_test_cluster_harness::model_card::nomic_embed_text_v1_5::nomic_embed_text_v1_5;
use paddler_tests::active_embedding_request_fixture::ActiveEmbeddingRequestFixture;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::llama_runtime::LlamaRuntime;

#[test]
fn an_embedding_batch_stopped_before_its_first_batch_produces_no_embeddings() {
    let runtime = LlamaRuntime::load(
        &nomic_embed_text_v1_5()
            .reference
            .cached_path(&Cache::from_env())
            .expect("the nomic embedding model must be in the Hugging Face cache"),
        agent_inference_runtime_context(1),
    )
    .expect("the nomic embedding model must load");
    let mut llama_context = runtime
        .create_embedding_llama_context(1, PoolingType::Mean)
        .expect("the embedding context must be created");
    let mut batch = LlamaBatch::new(runtime.model_runtime_parameters.n_batch.capacity(), 1);
    let ActiveEmbeddingRequestFixture {
        active_embedding_request,
        generate_embedding_stop_tx,
        mut generated_embedding_rx,
    } = ActiveEmbeddingRequestFixture::new(
        vec![EmbeddingInputTokenized {
            id: "document".to_owned(),
            tokens: vec![LlamaToken::new(0)],
        }],
        runtime
            .inference_runtime_context
            .slot_aggregated_status
            .clone(),
        NonZeroU16::MIN,
    );

    generate_embedding_stop_tx
        .send(())
        .expect("the stop request must reach the embedding batch");

    let continuing_requests = EmbeddingBatchProcessor {
        batch: &mut batch,
        llama_context: &mut llama_context,
        normalization: EmbeddingNormalization,
        scheduler_context: &EmbeddingSchedulerContext {
            agent_name: None,
            desired_slots_total: 1,
            n_batch: runtime.model_runtime_parameters.n_batch.tokens_usize(),
            pooling_type: PoolingType::Mean,
        },
    }
    .embed_next_batch(vec![active_embedding_request]);

    assert_eq!(continuing_requests.len(), 0);
    assert_eq!(
        generated_embedding_rx.try_recv(),
        Ok(EmbeddingResult::NoEmbeddingsProduced)
    );
    assert_eq!(
        generated_embedding_rx.try_recv(),
        Err(TryRecvError::Disconnected)
    );
}
