use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::request_params::GenerateTokensParams;

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateTokensChannel {
    pub chunk_sender: mpsc::Sender<String>,
    pub params: GenerateTokensParams,
}
