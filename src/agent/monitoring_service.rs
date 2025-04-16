use actix_web::web::Bytes;
use async_trait::async_trait;
use log::{debug, error};
use pingora::{server::ShutdownWatch, services::Service};
use std::net::SocketAddr;
use tokio::{
    sync::broadcast::Sender,
    time::{interval, Duration, MissedTickBehavior},
};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{
    balancer::status_update::StatusUpdate, errors::result::Result,
    llamacpp::llamacpp_client::LlamacppClient,
};

pub struct MonitoringService {
    pub external_llamacpp_addr: SocketAddr,
    pub llamacpp_client: LlamacppClient,
    pub monitoring_interval: Duration,
    pub name: Option<String>,
    pub status_update_tx: Sender<Bytes>,
}

impl std::fmt::Debug for MonitoringService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MonitoringService {{ external_llamacpp_addr: {} }}",
            self.external_llamacpp_addr
        )
    }
}

impl MonitoringService {
    pub fn new(
        external_llamacpp_addr: SocketAddr,
        llamacpp_client: LlamacppClient,
        monitoring_interval: Duration,
        name: Option<String>,
        status_update_tx: Sender<Bytes>,
    ) -> Result<Self> {
        Ok(MonitoringService {
            external_llamacpp_addr,
            llamacpp_client,
            monitoring_interval,
            name,
            status_update_tx,
        })
    }

    pub async fn fetch_status(&self) -> Result<StatusUpdate> {
        match self.llamacpp_client.get_available_slots().await {
            Ok(slots_response) => Ok(StatusUpdate::new(
                self.name.to_owned(),
                None,
                self.external_llamacpp_addr.to_owned(),
                slots_response.is_authorized,
                slots_response.is_slot_endpoint_enabled,
                slots_response.slots,
            )),
            Err(err) => Ok(StatusUpdate::new(
                self.name.to_owned(),
                Some(err.to_string()),
                self.external_llamacpp_addr.to_owned(),
                None,
                None,
                vec![],
            )),
        }
    }

    pub async fn report_status(&self, status: StatusUpdate) -> Result<usize> {
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
        let mut ticker = interval(self.monitoring_interval);

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down monitoring service");
                    return;
                },
                _ = ticker.tick() => {
                    match self.fetch_status().await {
                        Ok(status) => {
                            if let Err(err) = self.report_status(status).await {
                                error!("Failed to report status: {}", err);
                            }
                        }
                        Err(err) => {
                            error!("Failed to fetch status: {}", err);
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
