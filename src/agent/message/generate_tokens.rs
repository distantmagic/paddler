use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc;

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateTokens {
    pub chunk_sender: mpsc::Sender<String>,
    pub max_tokens: i32,
    pub prompt: String,
}
