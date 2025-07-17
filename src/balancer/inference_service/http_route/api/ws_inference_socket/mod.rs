pub mod jsonrpc;

use std::sync::Arc;

use actix_web::get;
use actix_web::rt;
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

struct InferenceSocketControllerContext {
    agent_controller_pool: Data<AgentControllerPool>,
}

struct InferenceSocketController {
    agent_controller_pool: Data<AgentControllerPool>,
}

#[async_trait]
impl ControlsWebSocketEndpoint for InferenceSocketController {
    type Context = InferenceSocketControllerContext;
    type Message = InferenceJsonRpcMessage;

    fn create_context(&self) -> Self::Context {
        InferenceSocketControllerContext {
            agent_controller_pool: self.agent_controller_pool.clone(),
        }
    }

    async fn handle_deserialized_message(
        context: Arc<Self::Context>,
        deserialized_message: Self::Message,
        mut session: Session,
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
                request: InferenceJsonRpcRequest::GenerateTokens(params),
            }) => {
                rt::spawn(async move {
                    println!("Received GenerateTokens request");
                    if let Some(agent_controller) =
                        context.agent_controller_pool.find_best_agent_controller()
                    {
                        // session.text("xd").await?;
                        // println!("Found agent controller: {:?}", agent_controller.name);
                    } else {
                        println!("No agent controller found");
                    }
                });

                return Ok(ContinuationDecision::Continue);
            }
        }
    }
}

#[get("/api/v1/inference_socket")]
async fn respond(
    agent_controller_pool: Data<AgentControllerPool>,
    payload: Payload,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let inference_socket_controller = InferenceSocketController {
        agent_controller_pool,
    };

    inference_socket_controller.respond(payload, req)
}
