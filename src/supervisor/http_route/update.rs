use actix_web::{
    post,
    web::{self},
    HttpResponse,
};
use tokio::sync::broadcast::Sender;

use crate::{errors::result::Result, supervisor::llamacpp_configuration::LlamacppConfiguration};

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("v1/params")]
async fn respond(
    status_update_tx: web::Data<Sender<LlamacppConfiguration>>,
    config: web::Json<LlamacppConfiguration>,
) -> Result<HttpResponse> {
    status_update_tx.send(config.0)?;

    Ok(HttpResponse::Ok().finish())
}
