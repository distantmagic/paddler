use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc::Sender;

#[derive(Message)]
#[rtype(result = "Result<String>")]
pub struct Generate {
    pub chunk_sender: Sender<String>,
    pub max_tokens: i32,
    pub prompt: String,
}
