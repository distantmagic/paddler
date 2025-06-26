use actix_web::post;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;

use crate::supervisor::change_request::ChangeRequest;
use crate::supervisor::reconciliation_queue::ReconciliationQueue;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("/api/v1/change_request")]
async fn respond(
    change_request: web::Json<ChangeRequest>,
    reconciliation_queue: web::Data<ReconciliationQueue>,
) -> Result<HttpResponse, Error> {
    reconciliation_queue
        .register_change_request(change_request.into_inner())
        .await?;

    Ok(HttpResponse::Accepted().finish())
}
