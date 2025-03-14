use std::time::Instant;

use actix_web::{post, web, HttpResponse};
use mavec::error::MavecError;
use serde_json::Value;

use crate::{
    errors::{app_error::AppError, result::Result},
    supervisor::{debounce::handle_throttle, management_service::State},
};

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("v1/params")]
pub async fn respond(config: web::Json<Value>, state: web::Data<State>) -> Result<HttpResponse> {
    let _ = handle_throttle(state.clone());

    let now = Instant::now();

    let mut last_request_time = state.last_request.lock()?;
    let mut args_vec = state.args.lock()?;
    let mut throttle = state.throttle.lock()?;

    config
        .0
        .as_object()
        .and_then(|object| object.get("args"))
        .and_then(|input_args| input_args.as_object())
        .map(|args| {
            args_vec.push(args.clone());
            let _ = throttle.accept().ok();
            *last_request_time = Some(now);
        })
        .ok_or_else(|| {
            AppError::MapToVecParseError(MavecError::JsonStructureParseError(
                "Could not parse structure as an object".to_string(),
            ))
        })?;

    Ok(HttpResponse::Ok().finish())
}
