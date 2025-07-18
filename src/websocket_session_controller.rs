use std::marker::PhantomData;

use actix_ws::Session;
use anyhow::Result;
use serde::Serialize;

use crate::rpc_message::RpcMessage;

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

    pub async fn send_response(&mut self, message: TResponse) -> Result<()> {
        let serialized_message = serde_json::to_string(&message)?;

        self.session.text(serialized_message).await?;

        Ok(())
    }
}
