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
use tokio::sync::broadcast;

use self::jsonrpc::notification_params::GenerateTokensParams;
use self::jsonrpc::Notification as InferenceJsonRpcNotification;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::controls_websocket_endpoint::ContinuationDecision;
use crate::controls_websocket_endpoint::ControlsWebSocketEndpoint;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

struct InferenceSocketController {}

#[async_trait]
impl ControlsWebSocketEndpoint for InferenceSocketController {
    type Context = ();
    type Message = InferenceJsonRpcNotification;

    fn create_context(&self) -> Self::Context {}

    async fn handle_deserialized_message(
        _context: Arc<Self::Context>,
        deserialized_message: Self::Message,
        _session: Session,
        _shutdown_tx: broadcast::Sender<()>,
    ) -> Result<ContinuationDecision> {
        match deserialized_message {
            InferenceJsonRpcNotification::GenerateTokens(GenerateTokensParams {
                prompt,
            }) => {
                println!("Received GenerateTokens request with prompt: {prompt}");

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
