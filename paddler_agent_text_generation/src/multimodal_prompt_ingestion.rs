use std::mem;
use std::sync::Arc;

use llama_cpp_bindings::SampledToken;
use llama_cpp_bindings::SampledTokenClassifier;
use llama_cpp_bindings::context::LlamaContext;
use llama_cpp_bindings::error::EvalMultimodalChunksError;
use llama_cpp_bindings::llama_batch::LlamaBatch;
use llama_cpp_bindings::mtmd::FittedMtmdInputChunk;
use llama_cpp_bindings::mtmd::FittedMtmdInputChunks;
use llama_cpp_bindings::mtmd::MtmdInputChunks;

use paddler_agent_runtime::sequence_id::SequenceId;

use crate::leading_bos_ingestion::LeadingBosIngestion;
use crate::multimodal_ingestion_progress::MultimodalIngestionProgress;
use crate::resident_multimodal_context::ResidentMultimodalContext;
use crate::text_generation_error::TextGenerationError;

#[derive(Debug)]
pub struct MultimodalPromptIngestion {
    pub chunks: FittedMtmdInputChunks<MtmdInputChunks>,
    pub leading_bos_ingestion: LeadingBosIngestion,
    pub multimodal_context: Arc<ResidentMultimodalContext>,
    pub n_batch: i32,
    pub next_chunk_index: usize,
}

impl MultimodalPromptIngestion {
    pub fn ingest_next_chunk(
        &mut self,
        token_classifier: &mut SampledTokenClassifier<'_>,
        llama_context: &mut LlamaContext<'_>,
        batch: &mut LlamaBatch<'static>,
        sequence_id: SequenceId,
        start_position: i32,
    ) -> Result<MultimodalIngestionProgress, TextGenerationError> {
        if let LeadingBosIngestion::Ingest(bos_token) =
            mem::replace(&mut self.leading_bos_ingestion, LeadingBosIngestion::Skip)
        {
            batch.clear();
            batch
                .add(
                    &SampledToken::Content(bos_token),
                    start_position,
                    &[sequence_id.llama_sequence_id()],
                    false,
                )
                .map_err(TextGenerationError::BatchAssemblyFailed)?;
            llama_context
                .decode(batch)
                .map_err(TextGenerationError::LeadingBosDecodeFailed)?;
            token_classifier.record_prompt_tokens(1);
            token_classifier.ingest_prompt_tokens(&[bos_token]);

            return Ok(MultimodalIngestionProgress::ChunksRemain {
                next_position: start_position + 1,
            });
        }

        self.ingest_next_fitted_chunk(token_classifier, llama_context, sequence_id, start_position)
            .map_err(TextGenerationError::MultimodalIngestionFailed)
    }

    fn ingest_next_fitted_chunk(
        &mut self,
        token_classifier: &mut SampledTokenClassifier<'_>,
        llama_context: &LlamaContext<'_>,
        sequence_id: SequenceId,
        start_position: i32,
    ) -> Result<MultimodalIngestionProgress, EvalMultimodalChunksError> {
        let chunk_index = self.next_chunk_index;
        let chunk = FittedMtmdInputChunk::at(&self.chunks, chunk_index)
            .ok_or(EvalMultimodalChunksError::ChunkOutOfBounds(chunk_index))?;
        let is_last_chunk = chunk_index + 1 == self.chunks.len();
        let next_position = chunk.eval_single(
            &self.multimodal_context,
            llama_context,
            start_position,
            sequence_id.llama_sequence_id(),
            self.n_batch,
            is_last_chunk,
        )?;

        token_classifier.ingest_fitted_chunk(&chunk);

        self.next_chunk_index += 1;

        Ok(if is_last_chunk {
            MultimodalIngestionProgress::PromptIngested { next_position }
        } else {
            MultimodalIngestionProgress::ChunksRemain { next_position }
        })
    }
}
