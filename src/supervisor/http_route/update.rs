use actix_web::{
    post,
    web::{self},
    HttpResponse,
};
use mavec::{core::to_vec, error::MavecError};
use serde_json::Value;
use tokio::sync::broadcast::Sender;

use crate::{
    errors::{app_error::AppError, result::Result},
    supervisor::config::Config,
};

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("v1/params")]
async fn respond(
    update_llamacpp_tx: web::Data<Sender<Vec<String>>>,
    config: web::Json<Value>,
) -> Result<HttpResponse> {
    match config.0.as_object() {
        Some(object) => match object["args"].as_object() {
            Some(args) => {
                let args = Config(to_vec(Value::Object(args.clone()))?.into()).to_llamacpp_arg()?;
                update_llamacpp_tx.send(args)?;
            }
            None => {
                return Err(AppError::MapToVecParseError(
                    MavecError::JsonStructureParseError(
                        "Could not parse structure as an object".to_string(),
                    ),
                ));
            }
        },
        None => {
            return Err(AppError::MapToVecParseError(
                MavecError::JsonStructureParseError(
                    "Could not parse args structure as an object".to_string(),
                ),
            ));
        }
    }

    Ok(HttpResponse::Ok().finish())
}
