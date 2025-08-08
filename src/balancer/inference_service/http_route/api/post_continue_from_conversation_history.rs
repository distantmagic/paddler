use actix_web::http::header;
use actix_web::post;
use actix_web::rt;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::StreamExt;
use log::error;
use nanoid::nanoid;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::balancer::inference_service::app_data::AppData;
use crate::balancer::inference_service::chunk_forwarding_session_controller::ChunkForwardingSessionController;
use crate::balancer::inference_service::controls_inference_endpoint::ControlsInferenceEndpoint;
use crate::request_params::ContinueFromConversationHistoryParams;
use crate::request_params::continue_from_conversation_history_params::tool::tool_params::function_call::parameters_schema::raw_parameters_schema::RawParametersSchema;

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

    rt::spawn(async move {
        if let Err(err) = Controller::request_from_agent(
            app_data.buffered_request_manager.clone(),
            connection_close_tx,
            app_data.inference_service_configuration.clone(),
            match params.into_inner().validate() {
                Ok(validated_params) => validated_params,
                Err(validation_error) => {
                    error!("Params validation error: {validation_error}");

                    return;
                }
            },
            request_id,
            ChunkForwardingSessionController { chunk_tx },
        )
        .await
        {
            error!("Failed to handle request: {err}");
        }
    });

    let stream = UnboundedReceiverStream::new(chunk_rx)
        .map(|chunk: String| Ok::<_, Error>(Bytes::from(format!("{chunk}\n"))));

    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType::json())
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .streaming(stream))
}
