use std::fmt::Debug;
use std::sync::Arc;

use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::http::header;
use actix_web::rt;
use bytes::Bytes;
use futures::stream::StreamExt;
use log::error;
use nanoid::nanoid;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::agent::jsonrpc::Request as AgentJsonRpcRequest;
use crate::balancer::agent_controller::AgentController;
use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::balancer::chunk_forwarding_session_controller::ChunkForwardingSessionController;
use crate::balancer::chunk_forwarding_session_controller::transforms_outgoing_message::TransformsOutgoingMessage;
use crate::balancer::handles_agent_streaming_response::HandlesAgentStreamingResponse;
use crate::balancer::inference_client::Message as OutgoingMessage;
use crate::balancer::inference_client::Response as OutgoingResponse;
use crate::balancer::inference_service::configuration::Configuration as InferenceServiceConfiguration;
use crate::balancer::manages_senders::ManagesSenders;
use crate::balancer::request_from_agent::request_from_agent;
use crate::controls_session::ControlsSession as _;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::ErrorEnvelope;
use crate::streamable_result::StreamableResult;

pub async fn stream_from_agent<TParams, TTransformsOutgoingMessage>(
    buffered_request_manager: Arc<BufferedRequestManager>,
    inference_service_configuration: InferenceServiceConfiguration,
    params: TParams,
    transformer: TTransformsOutgoingMessage,
) -> Result<impl Responder, Error>
where
    TParams: Debug + Into<AgentJsonRpcRequest> + Send + 'static,
    AgentController: HandlesAgentStreamingResponse<TParams>,
    <<AgentController as HandlesAgentStreamingResponse<TParams>>::SenderCollection as ManagesSenders>::Value: Debug + Into<OutgoingResponse> + StreamableResult,
    TTransformsOutgoingMessage: Clone + TransformsOutgoingMessage + Send + Sync + 'static,
{
    let request_id: String = nanoid!();
    let (connection_close_tx, _connection_close_rx) = broadcast::channel(1);
    let (chunk_tx, chunk_rx) = mpsc::unbounded_channel();

    rt::spawn(async move {
        let mut session_controller = ChunkForwardingSessionController::new(chunk_tx, transformer);

        if let Err(err) = request_from_agent(
            buffered_request_manager.clone(),
            connection_close_tx,
            inference_service_configuration.clone(),
            params,
            request_id.clone(),
            session_controller.clone(),
        )
        .await
        {
            error!("Failed to handle request: {err}");

            session_controller
                .send_response_safe(OutgoingMessage::Error(ErrorEnvelope {
                    request_id: request_id.clone(),
                    error: JsonRpcError {
                        code: 500,
                        description: format!("Request {request_id} failed: {err}"),
                    },
                }))
                .await;
        }
    });

    let stream = UnboundedReceiverStream::new(chunk_rx)
        .map(|chunk: String| Ok::<_, Error>(Bytes::from(format!("{chunk}\n"))));

    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType::json())
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .streaming(stream))
}
