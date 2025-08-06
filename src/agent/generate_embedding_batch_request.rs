use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::embedding_result::EmbeddingResult;
use crate::request_params::GenerateEmbeddingBatchParams;

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateEmbeddingBatchRequest {
    pub generate_embedding_stop_rx: mpsc::UnboundedReceiver<()>,
    pub generated_embedding_tx: mpsc::UnboundedSender<EmbeddingResult>,
    pub params: GenerateEmbeddingBatchParams,
}
