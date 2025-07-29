use std::error::Error;

use actix_web::get;
use actix_web::web::ServiceConfig;
use actix_web::Responder;

pub fn register(cfg: &mut ServiceConfig) {
    cfg.service(respond);
}

#[get("/health")]
async fn respond() -> Result<impl Responder, Box<dyn Error>> {
    Ok("OK")
}
