use std::convert::Infallible;
use std::time::Duration;

use actix_web::Error;
use tokio::sync::broadcast;
use actix_web::rt;
use actix_web::Responder;
use actix_web::post;
use actix_web::web;
use actix_web_lab::sse;
use log::error;
use async_trait::async_trait;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::request_params::ContinueFromRawPromptParams;
use crate::balancer::inference_service::app_data::AppData;
use crate::balancer::inference_service::controls_inference_endpoint::ControlsInferenceEndpoint;
use crate::session_controller::SessionController;
use crate::balancer::inference_service::http_route::api::ws_inference_socket::client::Message as OutgoingMessage;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

struct ForwardingSessionController {
    sse_event_tx: mpsc::UnboundedSender<sse::Event>,
}

#[async_trait]
impl SessionController<OutgoingMessage> for ForwardingSessionController {
    async fn send_response(&mut self, message: OutgoingMessage) -> anyhow::Result<()> {
        self.sse_event_tx.send(
            sse::Event::Data(sse::Data::new(
                serde_json::to_string(&message)?
            ))
        )?;

        Ok(())
    }
}

struct ContinueFromRawPromptController {}

#[async_trait]
impl ControlsInferenceEndpoint for ContinueFromRawPromptController {
    type SessionController = ForwardingSessionController;
}

#[post("/api/v1/continue_from_raw_prompt")]
async fn respond(
    app_data: web::Data<AppData>,
    params: web::Json<ContinueFromRawPromptParams>,
) -> Result<impl Responder, Error> {
    let request_id: String = Uuid::new_v4().into();
    let (connection_close_tx, mut connection_close_rx) = broadcast::channel(1);
    let (sse_event_tx, mut sse_event_rx) = mpsc::unbounded_channel();

    rt::spawn(async move {
        if let Err(err) = ContinueFromRawPromptController::continue_from_raw_prompt(
            app_data.buffered_request_manager.clone(),
            connection_close_tx,
            app_data.inference_service_configuration.clone(),
            params.into_inner(),
            request_id,
            ForwardingSessionController {
                sse_event_tx,
            },
        ).await {
            error!("Failed to handle request: {err}");
        }
    });

    let event_stream = async_stream::stream! {
        loop {
            tokio::select! {
                _ = connection_close_rx.recv() => {
                    break;
                }
                event = sse_event_rx.recv() => {
                    match event {
                        Some(event) => yield Ok::<_, Infallible>(event),
                        None => break,
                    }
                }
            }
        }
    };

    Ok(sse::Sse::from_stream(event_stream).with_keep_alive(Duration::from_secs(10)))
}
