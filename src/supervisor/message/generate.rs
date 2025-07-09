use actix::Message;
use anyhow::Result;

#[derive(Message)]
#[rtype(result = "Result<String>")]
pub struct Generate {
    pub prompt: String,
}
