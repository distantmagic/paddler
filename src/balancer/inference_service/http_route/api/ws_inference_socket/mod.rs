pub mod client;
pub mod jsonrpc;
use std::sync::Arc;

use actix_web::get;
use actix_web::rt;
use actix_web::web::Payload;
use actix_web::web::ServiceConfig;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use anyhow::Result;
use async_trait::async_trait;
use log::debug;
use log::error;
use tokio::sync::broadcast;

use self::client::Message as OutgoingMessage;
use self::jsonrpc::Message as InferenceJsonRpcMessage;
use self::jsonrpc::Request as InferenceJsonRpcRequest;
use crate::controls_websocket_endpoint::ContinuationDecision;
use crate::controls_websocket_endpoint::ControlsWebSocketEndpoint;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::RequestEnvelope;
use crate::websocket_session_controller::WebSocketSessionController;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

struct InferenceSocketController {}

#[async_trait]
impl ControlsWebSocketEndpoint for InferenceSocketController {
    type Context = ();
    type IncomingMessage = InferenceJsonRpcMessage;
    type OutgoingMessage = OutgoingMessage;

    fn create_context(&self) -> Self::Context {}

    async fn handle_deserialized_message(
        connection_close_tx: broadcast::Sender<()>,
        _context: Arc<Self::Context>,
        deserialized_message: Self::IncomingMessage,
        mut _websocket_session_controller: WebSocketSessionController<Self::OutgoingMessage>,
    ) -> Result<ContinuationDecision> {
        match deserialized_message {
            InferenceJsonRpcMessage::Error(JsonRpcError { code, description }) => {
                error!("Received error from client: code: {code}, description: {description:?}");

                return Ok(ContinuationDecision::Continue);
            }
            InferenceJsonRpcMessage::Request(RequestEnvelope {
                id,
                request: InferenceJsonRpcRequest::GenerateTokens(params),
            }) => {
                debug!("Received GenerateTokens request from client: {id:?}, params: {params:?}");

                let mut connection_close_rx = connection_close_tx.subscribe();

                rt::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = connection_close_rx.recv() => {
                                debug!("Connection close signal received, stopping GenerateTokens loop.");
                                break;
                            }
                        }
                    }
                });

                return Ok(ContinuationDecision::Continue);
            }
        }
    }
}

#[get("/api/v1/inference_socket")]
async fn respond(payload: Payload, req: HttpRequest) -> Result<HttpResponse, Error> {
    let inference_socket_controller = InferenceSocketController {};

    inference_socket_controller.respond(payload, req)
}
