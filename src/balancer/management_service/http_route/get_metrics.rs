use std::error::Error;

use actix_web::get;
use actix_web::web::Data;
use actix_web::web::ServiceConfig;
use actix_web::HttpResponse;
use actix_web::Responder;
use indoc::indoc;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::agent_controller_pool_total_slots::AgentControllerPoolTotalSlots;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

#[get("/metrics")]
async fn respond(
    agent_controller_pool: Data<AgentControllerPool>,
) -> Result<impl Responder, Box<dyn Error>> {
    let AgentControllerPoolTotalSlots {
        slots_processing,
        slots_total,
    } = agent_controller_pool.total_slots();
    let requests_buffered = agent_controller_pool.total_buffered_requests();

    let metrics_response = format!(
        indoc! {"
# HELP paddler_slots_processing Number of processing slots
# TYPE paddler_slots_processing gauge
paddler_slots_processing {}

# HELP paddler_slots_total Number of total slots
# TYPE paddler_slots_total gauge
paddler_slots_total {}

# HELP paddler_requests_buffered Number of buffered requests
# TYPE paddler_requests_buffered gauge
paddler_requests_buffered {}
"},
        slots_processing, slots_total, requests_buffered
    );

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8; escaping=values")
        .body(metrics_response))
}
