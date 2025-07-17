use anyhow::Result;
use async_trait::async_trait;

use crate::rpc_message::RpcMessage;

#[async_trait]
pub trait SendsRpcMessage {
    async fn send_rpc_message<TMessage: RpcMessage>(&self, message: TMessage) -> Result<()>;
}
