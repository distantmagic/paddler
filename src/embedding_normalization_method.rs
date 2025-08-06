use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EmbeddingNormalizationMethod {
    Euclidean,
    // MaxAbsolute,
    None,
    // PNorm,
}

impl EmbeddingNormalizationMethod {
    pub fn can_transform_to(&self, other: &EmbeddingNormalizationMethod) -> bool {
        match (self, other) {
            (EmbeddingNormalizationMethod::Euclidean, EmbeddingNormalizationMethod::Euclidean) => true,
            (EmbeddingNormalizationMethod::None, EmbeddingNormalizationMethod::Euclidean) => true,
            (EmbeddingNormalizationMethod::None, EmbeddingNormalizationMethod::None) => true,
            _ => false,
        }
    }
}
