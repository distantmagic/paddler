use actix::{fut::future::WrapFuture, Actor, AsyncContext, Context, Handler};
use log::error;
use serde_json::to_vec;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tokio_stream::wrappers::BroadcastStream;
use url::Url;

use crate::balancer::status_update::StatusUpdate;
use crate::errors::result::Result;

#[allow(dead_code)]
pub struct StateReporter {
    interval_running: Arc<Mutex<bool>>,
    stats_endpoint_url: String,
    status_update_tx: Arc<Sender<actix_web::web::Bytes>>,

    // channel is closed when the initial receiver is dropped
    // therefore, we need to keep the reference to the sender
    status_update_rx: Receiver<actix_web::web::Bytes>,
}

impl StateReporter {
    pub fn new(management_addr: Url) -> Result<Self> {
        let (tx, rx) = channel(1);

        Ok(Self {
            interval_running: Arc::new(Mutex::new(false)),
            stats_endpoint_url: management_addr.join("/status_update")?.to_string(),
            status_update_rx: rx,
            status_update_tx: Arc::new(tx),
        })
    }
}

impl Actor for StateReporter {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        let stats_endpoint_url = self.stats_endpoint_url.clone();
        let status_update_tx = self.status_update_tx.clone();
        let interval_running = self.interval_running.clone();

        ctx.run_interval(Duration::from_secs(1), move |actor, ctx| {
            let interval_running = interval_running.clone();
            let stats_endpoint_url = stats_endpoint_url.clone();
            let status_update_tx = status_update_tx.clone();

            let fut = async move {
                match interval_running.lock() {
                    Ok(mut interval_running) => {
                        if *interval_running {
                            return;
                        }

                        *interval_running = true;

                        drop(interval_running);
                    }
                    Err(err) => {
                        error!("Could not lock interval_running: {}", err);

                        return;
                    }
                };

                let rx = status_update_tx.subscribe();
                let stream = BroadcastStream::new(rx);
                let reqwest_body = reqwest::Body::wrap_stream(stream);

                let result = reqwest::Client::new()
                    .post(stats_endpoint_url)
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

                match interval_running.lock() {
                    Ok(mut interval_running) => {
                        *interval_running = false;
                    }
                    Err(err) => {
                        error!("Could not lock interval_running: {}", err);

                        return;
                    }
                };
            }
            .into_actor(actor);

            ctx.spawn(fut);
        });
    }
}

impl Handler<StatusUpdate> for StateReporter {
    type Result = ();

    fn handle(&mut self, msg: StatusUpdate, _ctx: &mut Context<Self>) {
        let bytes = match to_vec(&msg) {
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
