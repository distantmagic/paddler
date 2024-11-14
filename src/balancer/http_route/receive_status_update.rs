use actix_web::{post, web, Error, HttpResponse};
use futures_util::StreamExt as _;

use crate::balancer::status_update::StatusUpdate;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("/status_update")]
async fn respond(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    while let Some(chunk) = payload.next().await {
        match serde_json::from_slice::<StatusUpdate>(&chunk?) {
            Ok(status_update) => {
                println!("Received status update: {:?}", status_update);
            }
            Err(e) => {
                return Err(Error::from(e));
            }
        }
    }

    Ok(HttpResponse::Accepted().finish())
}
