use actix_web::web::Bytes;
use async_trait::async_trait;
use log::{debug, error, info};
use pingora::{server::ShutdownWatch, services::Service};
use std::net::SocketAddr;
use tokio::{
    sync::broadcast::Sender,
    time::{interval, Duration, MissedTickBehavior},
};
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{errors::result::Result, llamacpp::llamacpp_client::LlamacppClient};

pub struct ApplyingService {
    llamacpp_client: LlamacppClient,
    llama_server_path: String,
    monitoring_interval: Duration
}

impl ApplyingService {
    pub fn new(llamacpp_client: LlamacppClient, llama_server_path: String, monitoring_interval: Duration) -> Result<Self> {
        // let agent_id = Uuid::new_v4();

        Ok(ApplyingService {
            llamacpp_client,
            llama_server_path,
            monitoring_interval,
        })
    }

    // async fn keep_connection_alive(&self) -> Result<()> {
    //     let status_update_rx = self.status_update_tx.subscribe();
    //     let stream = BroadcastStream::new(status_update_rx);
    //     let reqwest_body = reqwest::Body::wrap_stream(stream);

    //     info!("Establishing connection with management server");

    //     match reqwest::Client::new()
    //         .post(self.stats_endpoint_url.to_owned())
    //         .body(reqwest_body)
    //         .send()
    //         .await
    //     {
    //         Ok(_) => {
    //             error!("Management server connection closed");

    //             Ok(())
    //         }
    //         Err(err) => Err(err.into()),
    //     }
    // }
}

#[async_trait]
impl Service for ApplyingService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        let mut ticker = interval(self.monitoring_interval);

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down reporting service");
                    return;
                },
                // _ = ticker.tick() => {
                //     eprintln!("{}", "Tick Passed")
                // }
            }
        }
    }

    fn name(&self) -> &str {
        "applying"
    }

    fn threads(&self) -> Option<usize> {
        None
    }
}
