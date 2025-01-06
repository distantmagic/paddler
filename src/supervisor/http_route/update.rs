use std::collections::VecDeque;

use actix_web::{
    post,
    web::{self},
    HttpResponse,
};
use serde_json::Value;
use tokio::sync::broadcast::Sender;

use crate::{errors::result::Result, supervisor::config::to_vec};

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("v1/params")]
async fn respond(
    status_update_tx: web::Data<Sender<Vec<String>>>,
    config: web::Json<Value>,
) -> Result<HttpResponse> {
    let args = to_vec(config.0)?.to_llamacpp_arg()?;
    status_update_tx.send(args)?;

    Ok(HttpResponse::Ok().finish())
}
