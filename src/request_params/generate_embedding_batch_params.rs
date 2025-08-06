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
    pub fn chunk_by_input_size(&self, chunk_size: usize) -> Vec<GenerateEmbeddingBatchParams> {
        let mut batches: Vec<GenerateEmbeddingBatchParams> = Vec::new();
        let mut current_batch: Vec<EmbeddingInputDocument> = Vec::new();
        let mut current_size: usize = 0;

        for input in &self.input_batch {
            let input_size = input.content.chars().count();

            current_batch.push(input.clone());
            current_size += input_size;

            if current_size + input_size > chunk_size {
                if !current_batch.is_empty() {
                    batches.push(GenerateEmbeddingBatchParams {
                        input_batch: current_batch.clone(),
                        normalization_method: self.normalization_method.clone(),
                    });
                }
                current_batch.clear();
                current_size = 0;
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_by_input_size() {
        let params = GenerateEmbeddingBatchParams {
            input_batch: vec![
                EmbeddingInputDocument {
                    content: "Hello".to_string(),
                    id: "1".to_string(),
                },
                EmbeddingInputDocument {
                    content: "World".to_string(),
                    id: "2".to_string(),
                },
                EmbeddingInputDocument {
                    content: "This is a test".to_string(),
                    id: "3".to_string(),
                },
            ],
            normalization_method: EmbeddingNormalizationMethod::None,
        };

        let batches = params.chunk_by_input_size(10);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].input_batch.len(), 2);
        assert_eq!(batches[0].input_batch[0].id, "1");
        assert_eq!(batches[0].input_batch[1].id, "2");
        assert_eq!(batches[1].input_batch.len(), 1);
        assert_eq!(batches[1].input_batch[0].id, "3");
    }
}
