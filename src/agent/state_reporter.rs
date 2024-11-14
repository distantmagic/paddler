use actix::{fut::future::WrapFuture, AsyncContext};
use log::error;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::balancer::status_update::StatusUpdate;
use crate::errors::result::Result;

#[allow(dead_code)]
pub struct StateReporter {
    stats_endpoint_url: String,
    status_update_rx: broadcast::Receiver<actix_web::web::Bytes>,
    status_update_tx: Arc<broadcast::Sender<actix_web::web::Bytes>>,
}

impl StateReporter {
    pub fn new(management_addr: url::Url) -> Result<Self> {
        let (tx, rx) = broadcast::channel(1);

        Ok(Self {
            stats_endpoint_url: management_addr.join("/stream")?.to_string(),
            status_update_rx: rx,
            status_update_tx: Arc::new(tx),
        })
    }
}

impl actix::Actor for StateReporter {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut actix::Context<Self>) {
        let stats_endpoint_url = self.stats_endpoint_url.clone();
        let status_update_tx = self.status_update_tx.clone();

        let fut = async move {
            let rx = status_update_tx.subscribe();
            let stream = tokio_stream::wrappers::BroadcastStream::new(rx);
            let reqwest_body = reqwest::Body::wrap_stream(stream);

            let result = reqwest::Client::new()
                .post(stats_endpoint_url.clone())
                .body(reqwest_body)
                .send()
                .await;

            match result {
                Ok(_) => {
                    error!("Management server connection closed");
                }
                Err(err) => {
                    error!("Management server error: {}", err);
                }
            }
        }
        .into_actor(self);

        ctx.wait(fut);
        ctx.run_later(std::time::Duration::from_secs(1), |act, ctx| {
            act.started(ctx);
        });
    }
}

impl actix::Handler<StatusUpdate> for StateReporter {
    type Result = ();

    fn handle(&mut self, msg: StatusUpdate, _ctx: &mut actix::Context<Self>) {
        let bytes = match serde_json::to_vec(&msg) {
            Ok(bytes) => bytes,
            Err(err) => {
                error!("Could not convert status update to bytes: {}", err);
                return;
            }
        };

        let actix_bytes = actix_web::web::Bytes::from(bytes);

        if let Err(err) = self.status_update_tx.send(actix_bytes) {
            error!("Could not send status update: {}", err);
        }
    }
}
