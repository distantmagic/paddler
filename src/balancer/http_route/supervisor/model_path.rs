use actix_web::{get, web, Error};
use tokio::sync::broadcast::Sender;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("/")]
async fn respond(status_update_tx: web::Data<Sender<String>>) -> Result<String, Error> {
    Ok("Working!".to_string())
}
