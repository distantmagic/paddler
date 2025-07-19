use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::request_params::GenerateTokensParams;
use crate::response::ChunkResponse;

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateTokensChannel {
    pub chunk_sender: mpsc::Sender<ChunkResponse>,
    pub params: GenerateTokensParams,
    pub should_stop: Arc<AtomicBool>,
}
