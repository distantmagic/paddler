use actix::{
    AsyncContext,
    fut::future::WrapFuture,
};
use log::error;
use std::sync::Arc;
use tokio::time;
use tokio::sync::broadcast;

use crate::balancer::status_update::StatusUpdate;

#[allow(dead_code)]
pub struct StateReporter {
    management_addr: url::Url,
    status_update_rx: broadcast::Receiver<actix_web::web::Bytes>,
    status_update_tx: Arc<broadcast::Sender<actix_web::web::Bytes>>,
}

impl StateReporter {
    pub fn new(management_addr: url::Url) -> Self {
        let (tx, rx) = broadcast::channel(1);

        Self {
            management_addr,
            status_update_rx: rx,
            status_update_tx: Arc::new(tx),
        }
    }
}

impl actix::Actor for StateReporter {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut actix::Context<Self>) {
        let management_addr = self.management_addr.clone();
        let status_update_tx = self.status_update_tx.clone();

        ctx.spawn(
            async move {
                let mut interval = time::interval(time::Duration::from_secs(1));

                loop {
                    interval.tick().await;

                    let rx = status_update_tx.subscribe();
                    let stream = tokio_stream::wrappers::BroadcastStream::new(rx);
                    let reqwest_body = reqwest::Body::wrap_stream(stream);

                    let result = reqwest::Client::new()
                        .post(management_addr.clone())
                        .body(reqwest_body)
                        .send()
                        .await
                    ;

                    match result {
                        Ok(_) => {
                            error!("Mangement server connection closed");
                        },
                        Err(err) => {
                            error!("Management server error: {}", err);
                        },
                    }
                }
            }
            .into_actor(self)
        );
    }
}

impl actix::Handler<StatusUpdate> for StateReporter {
    type Result = ();

    fn handle(&mut self, msg: StatusUpdate, _ctx: &mut actix::Context<Self>) {
        println!("Received status update: {:?}", msg);

        serde_json::to_vec(&msg)
            .map(|bytes| actix_web::web::Bytes::from(bytes))
            .map_err(|err| {
                eprintln!("Error: {}", err);
            })
            .and_then(|bytes| {
                self.status_update_tx.send(bytes).map_err(|err| {
                    eprintln!("Error: {}", err);
                })
            })
            .ok();
    }
}
