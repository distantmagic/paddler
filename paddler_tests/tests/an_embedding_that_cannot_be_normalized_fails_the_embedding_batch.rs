#![cfg(feature = "tests_that_use_llms")]

use std::num::NonZeroU16;

use hf_hub::Cache;
use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::token::LlamaToken;
use tokio::sync::mpsc::error::TryRecvError;

use paddler_agent_embeddings::embedding_batch_processor::EmbeddingBatchProcessor;
use paddler_agent_embeddings::embedding_error::EmbeddingError;
use paddler_agent_embeddings::embedding_input_tokenized::EmbeddingInputTokenized;
use paddler_agent_embeddings::embedding_scheduler_context::EmbeddingSchedulerContext;
use paddler_inference_parameters::pooling_type::PoolingType;
use paddler_messaging::embedding_result::EmbeddingResult;
use paddler_test_cluster_harness::model_card::nomic_embed_text_v1_5::nomic_embed_text_v1_5;
use paddler_tests::active_embedding_request_fixture::ActiveEmbeddingRequestFixture;
use paddler_tests::agent_inference_runtime_context::agent_inference_runtime_context;
use paddler_tests::llama_runtime::LlamaRuntime;
use paddler_tests::normalization_of_an_embedding_too_long_for_rms_norm::DIMENSIONS_TOO_MANY_FOR_RMS_NORM;
use paddler_tests::normalization_of_an_embedding_too_long_for_rms_norm::NormalizationOfAnEmbeddingTooLongForRmsNorm;

const REQUEST_SEQUENCES: NonZeroU16 = NonZeroU16::new(2).unwrap();

#[test]
fn an_embedding_that_cannot_be_normalized_fails_the_embedding_batch() {
    let runtime = LlamaRuntime::load(
        &nomic_embed_text_v1_5()
            .reference
            .cached_path(&Cache::from_env())
            .expect("the nomic embedding model must be in the Hugging Face cache"),
        agent_inference_runtime_context(REQUEST_SEQUENCES.get()),
    )
    .expect("the nomic embedding model must load");
    let mut llama_context = runtime
        .create_embedding_llama_context(u32::from(REQUEST_SEQUENCES.get()), PoolingType::Mean)
        .expect("the embedding context must be created");
    let mut batch = LlamaBatch::new(
        runtime.model_runtime_parameters.n_batch.capacity(),
        i32::from(REQUEST_SEQUENCES.get()),
    );
    let ActiveEmbeddingRequestFixture {
        active_embedding_request,
        generate_embedding_stop_tx: _generate_embedding_stop_tx,
        mut generated_embedding_rx,
    } = ActiveEmbeddingRequestFixture::new(
        vec![
            EmbeddingInputTokenized {
                id: "first".to_owned(),
                tokens: vec![LlamaToken::new(101), LlamaToken::new(102)],
            },
            EmbeddingInputTokenized {
                id: "second".to_owned(),
                tokens: vec![LlamaToken::new(101), LlamaToken::new(102)],
            },
        ],
        runtime
            .inference_runtime_context
            .slot_aggregated_status
            .clone(),
        REQUEST_SEQUENCES,
    );

    let continuing_requests = EmbeddingBatchProcessor {
        batch: &mut batch,
        llama_context: &mut llama_context,
        normalization: NormalizationOfAnEmbeddingTooLongForRmsNorm,
        scheduler_context: &EmbeddingSchedulerContext {
            agent_name: None,
            desired_slots_total: REQUEST_SEQUENCES.get(),
            n_batch: runtime.model_runtime_parameters.n_batch.tokens_usize(),
            pooling_type: PoolingType::Mean,
        },
    }
    .embed_next_batch(vec![active_embedding_request]);

    assert_eq!(continuing_requests.len(), 0);
    assert_eq!(
        generated_embedding_rx.try_recv(),
        Ok(EmbeddingResult::EmbeddingTooLongForRmsNormalization(
            EmbeddingError::EmbeddingTooLongForRmsNormalization {
                dimensions: DIMENSIONS_TOO_MANY_FOR_RMS_NORM,
                source: u16::try_from(DIMENSIONS_TOO_MANY_FOR_RMS_NORM)
                    .expect_err("the dimensions must exceed sixteen bits"),
            }
            .to_string()
        ))
    );
    assert_eq!(
        generated_embedding_rx.try_recv(),
        Err(TryRecvError::Disconnected)
    );
}
