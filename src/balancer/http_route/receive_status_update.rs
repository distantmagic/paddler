use actix_web::{post, web, Error, HttpResponse};
use futures_util::StreamExt as _;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("/stream")]
async fn respond(mut payload: web::Payload) -> Result<HttpResponse, Error> {
    while let Some(chunk) = payload.next().await {
        println!("Chunk: {:?}", chunk);
    }

    println!("Stream ended");

    Ok(HttpResponse::Ok().finish())
}
