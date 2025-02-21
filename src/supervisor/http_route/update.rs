use actix_web::{
    post,
    web::{self},
    HttpResponse,
};
use mavec::error::MavecError;
use serde_json::Value;

use crate::errors::{app_error::AppError, result::Result};

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("v1/config")]
async fn respond(config: web::Json<Value>) -> Result<HttpResponse> {
    match config.0.as_array() {
        Some(_array) => {
            // eprintln!("{:#?}", to_vec(Value::Array(array.to_vec()))?);
        }
        None => {
            return Err(AppError::MapToVecParseError(
                MavecError::JsonStructureParseError(
                    "Could not parse args structure as an array".to_string(),
                ),
            ));
        }
    }

    Ok(HttpResponse::Ok().finish())
}
