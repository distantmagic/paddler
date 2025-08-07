use serde::Deserialize;
use serde::Serialize;

use crate::embedding::Embedding;
use crate::streamable_result::StreamableResult;

#[derive(Debug, Deserialize, Serialize)]
pub enum EmbeddingResult {
    Done,
    Embedding(Embedding),
}

impl StreamableResult for EmbeddingResult {
    fn is_done(&self) -> bool {
        matches!(self, EmbeddingResult::Done)
    }
}
