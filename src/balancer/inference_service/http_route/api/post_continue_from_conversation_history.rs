use actix_web::http::header;
use log::error;
use actix_web::post;
use actix_web::error::ErrorBadRequest;
use actix_web::rt;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::StreamExt;
use nanoid::nanoid;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::validates::Validates as _;
use crate::balancer::inference_service::app_data::AppData;
use crate::balancer::inference_service::chunk_forwarding_session_controller::ChunkForwardingSessionController;
use crate::balancer::inference_service::controls_inference_endpoint::ControlsInferenceEndpoint;
use crate::request_params::ContinueFromConversationHistoryParams;
use crate::request_params::continue_from_conversation_history_params::tool::tool_params::function_call::parameters_schema::raw_parameters_schema::RawParametersSchema;
use crate::jsonrpc::Error as JsonRpcError;
use crate::session_controller::SessionController as _;
use crate::jsonrpc::ErrorEnvelope;
use crate::balancer::inference_service::http_route::api::ws_inference_socket::client::Message as OutgoingMessage;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

struct Controller {}

#[async_trait]
impl ControlsInferenceEndpoint for Controller {
    type SessionController = ChunkForwardingSessionController;
}

#[post("/api/v1/continue_from_conversation_history")]
async fn respond(
    app_data: web::Data<AppData>,
    params: web::Json<ContinueFromConversationHistoryParams<RawParametersSchema>>,
) -> Result<impl Responder, Error> {
    let request_id: String = nanoid!();
    let (connection_close_tx, _connection_close_rx) = broadcast::channel(1);
    let (chunk_tx, chunk_rx) = mpsc::unbounded_channel();

    let validated_params = match params.into_inner().validate() {
        Ok(validated_params) => validated_params,
        Err(validation_error) => {
            return Err(ErrorBadRequest(format!(
                "Invalid request parameters: {validation_error}"
            )));
        }
    };

    rt::spawn(async move {
        let mut session_controller = ChunkForwardingSessionController { chunk_tx };

        if let Err(err) = Controller::request_from_agent(
            app_data.buffered_request_manager.clone(),
            connection_close_tx,
            app_data.inference_service_configuration.clone(),
            validated_params,
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
