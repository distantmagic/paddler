pub mod jsonrpc;

use std::sync::Arc;

use actix_web::get;
use actix_web::web::Data;
use actix_web::web::Payload;
use actix_web::web::ServiceConfig;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_ws::Session;
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::broadcast;

use self::jsonrpc::request_params::GenerateTokensParams;
use self::jsonrpc::Message as InferenceJsonRpcMessage;
use self::jsonrpc::Request as InferenceJsonRpcRequest;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::controls_websocket_endpoint::ContinuationDecision;
use crate::controls_websocket_endpoint::ControlsWebSocketEndpoint;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::RequestEnvelope;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

struct InferenceSocketController {}

#[async_trait]
impl ControlsWebSocketEndpoint for InferenceSocketController {
    type Context = ();
    type Message = InferenceJsonRpcMessage;

    fn create_context(&self) -> Self::Context {}

    async fn handle_deserialized_message(
        _context: Arc<Self::Context>,
        deserialized_message: Self::Message,
        _session: Session,
        _shutdown_tx: broadcast::Sender<()>,
    ) -> Result<ContinuationDecision> {
        match deserialized_message {
            InferenceJsonRpcMessage::Error(JsonRpcError {
                code,
                description,
            }) => {
                error!("Received error from client: code: {code}, description: {description:?}");

                return Ok(ContinuationDecision::Continue);
            }
            InferenceJsonRpcMessage::Request(RequestEnvelope {
                id,
                request:
                    InferenceJsonRpcRequest::GenerateTokens(GenerateTokensParams {
                        prompt,
                    }),
            }) => {
                println!("Received GenerateTokens request with prompt: {id} {prompt}");

                return Ok(ContinuationDecision::Continue);
            }
        }
    }
}

#[get("/api/v1/inference_socket")]
async fn respond(
    _agent_controller_pool: Data<AgentControllerPool>,
    payload: Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let inference_socket_controller = InferenceSocketController {};

    inference_socket_controller.respond(payload, req)
}
