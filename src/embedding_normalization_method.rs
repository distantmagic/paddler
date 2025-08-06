use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EmbeddingNormalizationMethod {
    Euclidean,
    // MaxAbsolute,
    None,
    // PNorm,
}
