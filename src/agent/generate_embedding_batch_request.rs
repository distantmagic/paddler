use actix::Message;
use anyhow::Result;

use crate::request_params::GenerateEmbeddingBatchParams;

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateEmbeddingBatchRequest {
    pub params: GenerateEmbeddingBatchParams,
}
