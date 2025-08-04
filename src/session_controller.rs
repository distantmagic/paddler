use async_trait::async_trait;
use anyhow::Result;
use serde::Serialize;

use crate::rpc_message::RpcMessage;

#[async_trait]
pub trait SessionController <TResponse>: Send + Sync
where
    TResponse: RpcMessage + Send + Serialize + Sync,
{
    async fn send_response(&mut self, message: TResponse) -> Result<()>;
}
