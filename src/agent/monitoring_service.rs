use actix_web::web::Bytes;
use async_trait::async_trait;
use log::{debug, error};
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use tokio::sync::broadcast::Sender;
use tokio::time::{sleep, Duration};
use url::Url;
use uuid::Uuid;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::balancer::status_update::StatusUpdate;
use crate::errors::result::Result;
use crate::llamacpp::llamacpp_client::LlamacppClient;

pub struct MonitoringService {
    external_llamacpp_addr: Url,
    id: Uuid,
    llamacpp_client: LlamacppClient,
    name: Option<String>,
    status_update_tx: Sender<Bytes>,
}

impl MonitoringService {
    pub fn new(
        external_llamacpp_addr: Url,
        llamacpp_client: LlamacppClient,
        name: Option<String>,
        status_update_tx: Sender<Bytes>,
    ) -> Result<Self> {
        Ok(MonitoringService {
            external_llamacpp_addr,
            id: Uuid::new_v4(),
            llamacpp_client,
            name,
            status_update_tx,
        })
    }

    async fn fetch_status(&self) -> Result<StatusUpdate> {
        Ok(StatusUpdate::new(
            self.id,
            self.name.clone(),
            self.external_llamacpp_addr.clone(),
            self.llamacpp_client.get_available_slots().await?,
        ))
    }

    async fn report_status(&self, status: StatusUpdate) -> Result<usize> {
        let status = Bytes::from(serde_json::to_vec(&status)?);

        Ok(self.status_update_tx.send(status)?)
    }
}

#[async_trait]
impl Service for MonitoringService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down monitoring service");
                    return;
                },
                _ = sleep(Duration::from_secs(1)) => {
                    match self.fetch_status().await {
                        Ok(status) => {
                            if let Err(err) = self.report_status(status).await {
                                error!("Unable to report status: {}", err);
                            }
                        }
                        Err(err) => {
                            error!("Unable to fetch status: {}", err);
                        }
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "monitoring"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
