use std::marker::PhantomData;

use async_trait::async_trait;
use actix_ws::Session;
use anyhow::Result;
use serde::Serialize;

use crate::rpc_message::RpcMessage;
use crate::session_controller::SessionController;

pub struct WebSocketSessionController<TResponse>
where
    TResponse: RpcMessage + Send + Serialize + Sync,
{
    session: Session,
    _marker: PhantomData<TResponse>,
}

impl<TResponse> WebSocketSessionController<TResponse>
where
    TResponse: RpcMessage + Send + Serialize + Sync,
{
    pub fn new(session: Session) -> Self {
        WebSocketSessionController {
            session,
            _marker: PhantomData,
        }
    }
}


#[async_trait]
impl<TResponse> SessionController<TResponse> for WebSocketSessionController<TResponse>
where
    TResponse: RpcMessage + Send + Serialize + Sync,
{
    async fn send_response(&mut self, message: TResponse) -> Result<()> {
        let serialized_message = serde_json::to_string(&message)?;

        self.session.text(serialized_message).await?;

        Ok(())
    }
}
