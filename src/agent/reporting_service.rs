use actix_web::web::Bytes;
use async_trait::async_trait;
use log::{debug, error, warn};
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use tokio::sync::broadcast::Sender;
use tokio::time::{sleep, Duration};
use tokio_stream::wrappers::BroadcastStream;
use url::Url;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::errors::result::Result;

pub struct ReportingService {
    stats_endpoint_url: String,
    status_update_tx: Sender<Bytes>,
}

impl ReportingService {
    pub fn new(management_addr: Url, status_update_tx: Sender<Bytes>) -> Result<Self> {
        Ok(ReportingService {
            stats_endpoint_url: management_addr.join("/status_update")?.to_string(),
            status_update_tx,
        })
    }

    async fn keep_connection_alive(&self) -> Result<()> {
        let status_update_rx = self.status_update_tx.subscribe();
        let stream = BroadcastStream::new(status_update_rx);
        let reqwest_body = reqwest::Body::wrap_stream(stream);

        match reqwest::Client::new()
            .post(self.stats_endpoint_url.clone())
            .body(reqwest_body)
            .send()
            .await
        {
            Ok(_) => {
                warn!("Management server connection closed");

                Ok(())
            }
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl Service for ReportingService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down reportingservice");
                    return;
                },
                _ = sleep(Duration::from_secs(1)) => {
                    if let Err(err) = self.keep_connection_alive().await {
                        error!("ReportingService: keep_connection_alive failed: {}", err);
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "reporting"
    }

    fn threads(&self) -> Option<usize> {
        None
    }
}
