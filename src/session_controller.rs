use anyhow::Result;
use async_trait::async_trait;
use log::error;
use serde::Serialize;

use crate::rpc_message::RpcMessage;

#[async_trait]
pub trait SessionController<TResponse>: Send + Sync
where
    TResponse: RpcMessage + Send + Serialize + Sync + 'static,
{
    async fn send_response(&mut self, message: TResponse) -> Result<()>;

    async fn send_response_safe(&mut self, message: TResponse) {
        if let Err(err) = self.send_response(message).await {
            error!("Failed to send response: {err}");
        }
    }
}
