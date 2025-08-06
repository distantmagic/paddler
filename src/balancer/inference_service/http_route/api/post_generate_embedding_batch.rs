use actix_web::error::ErrorNotImplemented;
use actix_web::error::ErrorServiceUnavailable;
use actix_web::http::header;
use actix_web::post;
use actix_web::rt;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::Responder;
use bytes::Bytes;
use futures::stream::StreamExt;
use nanoid::nanoid;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::balancer::inference_service::app_data::AppData;
use crate::request_params::GenerateEmbeddingBatchParams;

const CHARACTERS_PER_TOKEN_APPROXIMATELY: usize = 3;

pub fn register(cfg: &mut web::ServiceConfig) {
    cfg.service(respond);
}

#[post("/api/v1/generate_embedding_batch")]
async fn respond(
    app_data: web::Data<AppData>,
    params: web::Json<GenerateEmbeddingBatchParams>,
) -> Result<impl Responder, Error> {
    let balancer_applicable_state_holder = app_data.balancer_applicable_state_holder.clone();
    let agent_desired_state = match balancer_applicable_state_holder.get_agent_desired_state() {
        Some(agent_desired_state) => agent_desired_state,
        None => {
            return Err(ErrorServiceUnavailable(
                "Balancer applicable state is not yet set",
            ))
        }
    };

    if !agent_desired_state.inference_parameters.enable_embeddings {
        return Err(ErrorNotImplemented(
            "Embedding generation is not enabled in the inference parameters",
        ));
    }

    let (connection_close_tx, mut connection_close_rx) = broadcast::channel::<()>(1);
    let (chunk_tx, chunk_rx) = mpsc::unbounded_channel();

    let request_batches = params.chunk_by_input_size(
        agent_desired_state.inference_parameters.batch_n_tokens
            * CHARACTERS_PER_TOKEN_APPROXIMATELY,
    );

    rt::spawn(async move {});

    let stream = UnboundedReceiverStream::new(chunk_rx)
        .map(|chunk: String| Ok::<_, Error>(Bytes::from(format!("{chunk}\n"))))
        .take_until(async move {
            connection_close_rx.recv().await.ok();
        });

    Ok(HttpResponse::Ok()
        .insert_header(header::ContentType::json())
        .insert_header((header::CACHE_CONTROL, "no-cache"))
        .streaming(stream))
}
