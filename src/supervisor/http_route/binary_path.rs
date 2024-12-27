use actix_web::{get, web, HttpResponse};

use crate::{cmd::supervisor::UpdateLlamacpp, errors::result::Result};

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[get("v1/binary/{path:.*}")]
async fn respond(
    status_update_tx: web::Data<UpdateLlamacpp>,
    path: web::Path<(String)>,
) -> Result<HttpResponse> {
    let binary_path = path.into_inner();

    status_update_tx.update_binary_tx.send(binary_path)?;

    Ok(HttpResponse::Ok().finish())
}
