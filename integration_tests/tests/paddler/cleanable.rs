use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Cleanable {
    async fn cleanup(&mut self) -> Result<()>;
}
