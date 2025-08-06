use serde::Deserialize;
use serde::Serialize;

use crate::embedding::Embedding;

#[derive(Debug, Deserialize, Serialize)]
pub enum EmbeddingResult {
    Done,
    Embedding(Embedding),
}
