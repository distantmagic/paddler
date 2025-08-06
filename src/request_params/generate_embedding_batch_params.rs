use serde::Deserialize;
use serde::Serialize;

use crate::embedding_input_document::EmbeddingInputDocument;
use crate::embedding_normalization_method::EmbeddingNormalizationMethod;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GenerateEmbeddingBatchParams {
    pub input_batch: Vec<EmbeddingInputDocument>,
    pub normalization_method: EmbeddingNormalizationMethod,
}

impl GenerateEmbeddingBatchParams {
    /// Input size is the total number of characters in the resulting batches.
    pub fn chunk_by_input_size(&self, input_size: usize) -> Vec<GenerateEmbeddingBatchParams> {
        let mut batches: Vec<GenerateEmbeddingBatchParams> = Vec::new();
        let mut current_batch: Vec<EmbeddingInputDocument> = Vec::new();
        let mut current_size: usize = 0;

        for input in &self.input_batch {
            let input_size = input.content.chars().count();

            if current_size + input_size > input_size {
                if !current_batch.is_empty() {
                    batches.push(GenerateEmbeddingBatchParams {
                        input_batch: current_batch.clone(),
                        normalization_method: self.normalization_method.clone(),
                    });
                }
                current_batch.clear();
                current_size = 0;
            }
            current_batch.push(input.clone());
            current_size += input_size;
        }

        if !current_batch.is_empty() {
            batches.push(GenerateEmbeddingBatchParams {
                input_batch: current_batch,
                normalization_method: self.normalization_method.clone(),
            });
        }

        batches
    }
}
