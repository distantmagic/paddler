use actix_web::{post, web::{self}, HttpResponse};
use tokio::sync::broadcast::Sender;

use crate::{errors::result::Result, supervisor::llamacpp_configuration::LlamacppConfiguration};

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("v1/params")]
async fn respond(
    status_update_tx: web::Data<Sender<String>>,
    config: web::Json<LlamacppConfiguration>,
) -> Result<HttpResponse> {
    let host = config.clone().get_host();
    let port = config.clone().get_port();

    eprintln!("{}:{}", host, port);

    // status_update_tx.send(binary_path)?;

    Ok(HttpResponse::Ok().finish())
}
