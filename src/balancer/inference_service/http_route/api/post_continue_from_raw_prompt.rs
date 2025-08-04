use std::convert::Infallible;
use std::time::Duration;

use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::get;
use actix_web::post;
use actix_web::web;
use actix_web_lab::sse;
use log::error;
use uuid::Uuid;

use crate::request_params::ContinueFromRawPromptParams;
use crate::balancer::management_service::app_data::AppData;
use crate::balancer_desired_state::BalancerDesiredState;
use crate::balancer::buffered_request_agent_wait_result::BufferedRequestAgentWaitResult;
use crate::jsonrpc::Error as JsonRpcError;
use crate::produces_snapshot::ProducesSnapshot as _;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("/api/v1/continue_from_raw_prompt")]
async fn respond(
    app_data: web::Data<AppData>,
    params: web::Json<ContinueFromRawPromptParams>,
) -> Result<impl Responder, Error> {
    let event_stream = async_stream::stream! {
        let send_event = |info| {
            match serde_json::to_string(&info) {
                Ok(json) => Some(Ok::<_, Infallible>(sse::Event::Data(sse::Data::new(json)))),
                Err(err) => {
                    error!("Failed to serialize generated tokens: {err}");
                    None
                }
            }
        };

        loop {
            match app_data.agent_controller_pool.make_snapshot() {
                Ok(agent_controller_pool_snapshot) => {
                    if let Some(event) = send_event(agent_controller_pool_snapshot) {
                        yield event;
                    }
                }
                Err(err) => error!("Failed to get agent controller pool snapshot: {err}"),
            }

            app_data.agent_controller_pool.update_notifier.notified().await;
        }
    };

    Ok(sse::Sse::from_stream(event_stream).with_keep_alive(Duration::from_secs(10)))
}
