use anyhow::anyhow;
use anyhow::Result;

use crate::embedding_normalization_method::EmbeddingNormalizationMethod;

pub struct Embedding {
    pub embedding: Vec<f32>,
    pub normalization_method: EmbeddingNormalizationMethod,
    pub source_document_id: String,
}

impl Embedding {
    pub fn normalize(self, normalization_method: &EmbeddingNormalizationMethod) -> Result<Self> {
        if !self.normalization_method.can_transform_to(normalization_method) {
            return Err(anyhow!(
                "Cannot transform from {:?} to {normalization_method:?}",
                self.normalization_method
            ));
        }

        if self.normalization_method == *normalization_method {
            return Ok(self);
        }

        Ok(Self {
            embedding: match normalization_method {
                EmbeddingNormalizationMethod::Euclidean => {
                    let magnitude = self.embedding
                        .iter()
                        .fold(0.0, |acc, &val| val.mul_add(val, acc))
                        .sqrt();

                    if magnitude == 0.0 {
                        vec![0.0; self.embedding.len()]
                    } else {
                        self.embedding.iter().map(|&val| val / magnitude).collect()
                    }
                }
                EmbeddingNormalizationMethod::None => self.embedding,
            },
            normalization_method: normalization_method.clone(),
            source_document_id: self.source_document_id.clone(),
        })
    }
}
