use actix_web::rt;
use actix_ws::Session;
use anyhow::Result;
use tokio::sync::mpsc::channel;

use super::handler::Handler;
use super::handler::SetState;
use super::request_params::SetStateParams;
use super::Request as JsonRpcRequest;
use super::Response as JsonRpcResponse;

const CONNECTION_MSG_BUFFER_SIZE: usize = 100;
const RESPONSE_INTERNAL_ERROR: &str =
    r#"{"id":null,"error":{"code":500,"message":"Internal error"}}"#;

pub struct HandlerCollection {
    pub set_state: SetState,
}

impl HandlerCollection {
    pub async fn dispatch(&self, request: JsonRpcRequest, mut session: Session) -> Result<()> {
        match request {
            JsonRpcRequest::SetState(params) => {
                let (tx, mut rx) = channel::<
                    JsonRpcResponse<<SetState as Handler<SetStateParams>>::ResponseResult>,
                >(CONNECTION_MSG_BUFFER_SIZE);

                rt::spawn(async move {
                    while let Some(response) = rx.recv().await {
                        let serialized = serde_json::to_string(&response)
                            .unwrap_or_else(|_| RESPONSE_INTERNAL_ERROR.to_string());

                        if session.text(serialized).await.is_err() {
                            break;
                        }
                    }
                });

                self.set_state.handle(tx, params).await
            }
        }
    }
}
