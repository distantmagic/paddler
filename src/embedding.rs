use anyhow::anyhow;
use anyhow::Result;

use crate::embedding_normalization_method::EmbeddingNormalizationMethod;

fn l2(embedding: &[f32]) -> Vec<f32> {
    let magnitude = embedding
        .iter()
        .fold(0.0, |acc, &val| val.mul_add(val, acc))
        .sqrt();

    if magnitude == 0.0 {
        return vec![0.0; embedding.len()];
    }

    embedding.iter().map(|&val| val / magnitude).collect()
}

fn rms_norm(embedding: &[f32], eps: f32) -> Vec<f32> {
    let mean_square = embedding
        .iter()
        .fold(0.0, |acc, &val| val.mul_add(val, acc))
        / embedding.len() as f32;

    let rms = (mean_square + eps).sqrt();

    if rms == 0.0 {
        return vec![0.0; embedding.len()];
    }

    embedding.iter().map(|&val| val / rms).collect()
}

pub struct Embedding {
    pub embedding: Vec<f32>,
    pub normalization_method: EmbeddingNormalizationMethod,
    pub source_document_id: String,
}

impl Embedding {
    pub fn normalize(self, normalization_method: &EmbeddingNormalizationMethod) -> Result<Self> {
        if !self
            .normalization_method
            .can_transform_to(normalization_method)
        {
            return Err(anyhow!(
                "Cannot transform from {:?} to {normalization_method:?}",
                self.normalization_method
            ));
        }

        if self
            .normalization_method
            .needs_transformation_to(normalization_method)
        {
            return Ok(self);
        }

        Ok(Self {
            embedding: match normalization_method {
                EmbeddingNormalizationMethod::None => self.embedding,
                EmbeddingNormalizationMethod::L2 => l2(&self.embedding),
                EmbeddingNormalizationMethod::RmsNorm { epsilon } => {
                    rms_norm(&self.embedding, *epsilon)
                }
            },
            normalization_method: normalization_method.clone(),
            source_document_id: self.source_document_id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_l2() {
        let embedding = vec![3.0, 4.0];
        let normalized = l2(&embedding);

        assert_eq!(normalized, vec![0.6, 0.8]);

        let zero_embedding = vec![0.0, 0.0];
        let normalized_zero = l2(&zero_embedding);

        assert_eq!(normalized_zero, vec![0.0, 0.0]);
    }
}
