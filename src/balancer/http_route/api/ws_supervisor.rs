use actix_web::get;
use actix_web::web;
use actix_web::web::Payload;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use tokio::time::Duration;

const MAX_CONCURRENT_HANDLERS_PER_CONNECTION: usize = 10;
const MAX_CONTINUATION_SIZE: usize = 100 * 1024;
const PING_INTERVAL: Duration = Duration::from_secs(3);

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/api/v1/supervisor")]
async fn respond(payload: Payload, req: HttpRequest) -> Result<HttpResponse, Error> {
    let (res, _session, msg_stream) = actix_ws::handle(&req, payload)?;

    let _aggregated_msg_stream = msg_stream
        .aggregate_continuations()
        .max_continuation_size(MAX_CONTINUATION_SIZE);

    Ok(res)
}
