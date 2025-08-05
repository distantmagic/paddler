use actix_web::Error;
use tokio::sync::broadcast;
use actix_web::rt;
use actix_web::Responder;
use actix_web::post;
use actix_web::web;
use actix_web::HttpResponse;
use log::error;
use bytes::Bytes;
use actix_web::http::header;
use futures::stream::StreamExt;
use async_trait::async_trait;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::request_params::ContinueFromRawPromptParams;
use crate::balancer::inference_service::app_data::AppData;
use crate::balancer::inference_service::controls_inference_endpoint::ControlsInferenceEndpoint;
use crate::balancer::inference_service::chunk_forwarding_session_controller::ChunkForwardingSessionController;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

struct ContinueFromRawPromptController {}

#[async_trait]
impl ControlsInferenceEndpoint for ContinueFromRawPromptController {
    type SessionController = ChunkForwardingSessionController;
}

#[post("/api/v1/continue_from_raw_prompt")]
async fn respond(
    app_data: web::Data<AppData>,
    params: web::Json<ContinueFromRawPromptParams>,
) -> Result<impl Responder, Error> {
    let request_id: String = Uuid::new_v4().into();
    let (connection_close_tx, mut connection_close_rx) = broadcast::channel(1);
    let (chunk_tx, chunk_rx) = mpsc::unbounded_channel();

    rt::spawn(async move {
        if let Err(err) = ContinueFromRawPromptController::continue_from_raw_prompt(
            app_data.buffered_request_manager.clone(),
            connection_close_tx,
            app_data.inference_service_configuration.clone(),
            params.into_inner(),
            request_id,
            ChunkForwardingSessionController {
                chunk_tx,
            },
        ).await {
            error!("Failed to handle request: {err}");
        }
    });

    let stream = UnboundedReceiverStream::new(chunk_rx)
        .map(|chunk: String| {
            Ok::<_, Error>(Bytes::from(format!("{chunk}\n")))
        })
        .take_until(async move {
            connection_close_rx.recv().await.ok();
        });

    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType::json())
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .streaming(stream))
}
