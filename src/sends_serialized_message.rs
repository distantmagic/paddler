use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait SendsSerializedMessage {
    async fn send_serialized<TMessage: Send + Serialize>(&self, message: TMessage) -> Result<()>;
}
