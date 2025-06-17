use std::net::SocketAddr;

use actix_web::web::Bytes;
use async_trait::async_trait;
use log::debug;
use log::info;
use log::error;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use tokio::sync::broadcast::Sender;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;

use crate::balancer::status_update::StatusUpdate;
use crate::errors::result::Result;
use crate::llamacpp::llamacpp_client::LlamacppClient;

pub struct MonitoringService {
    external_llamacpp_addr: SocketAddr,
    llamacpp_client: LlamacppClient,
    monitoring_interval: Duration,
    name: Option<String>,
    status_update_tx: Sender<Bytes>,
    check_model: bool, // Store the check_model flag
}

impl MonitoringService {
    pub fn new(
        external_llamacpp_addr: SocketAddr,
        llamacpp_client: LlamacppClient,
        monitoring_interval: Duration,
        name: Option<String>,
        status_update_tx: Sender<Bytes>,
        check_model: bool, // Include the check_model flag
    ) -> Result<Self> {
        Ok(MonitoringService {
            external_llamacpp_addr,
            llamacpp_client,
            monitoring_interval,
            name,
            status_update_tx,
            check_model,
        })
    }

    async fn fetch_status(&self) -> Result<StatusUpdate> {
        match self.llamacpp_client.get_available_slots().await {
            Ok(slots_response) => {
                let model = if self.check_model {
                    self.llamacpp_client.get_model().await?
                } else {
                    None
                };

                info!("Agent: {:?} Model: {:?}", self.name, model);

                Ok(StatusUpdate::new(
                    self.name.to_owned(),
                    None,
                    self.external_llamacpp_addr.to_owned(),
                    slots_response.is_authorized,
                    slots_response.is_slot_endpoint_enabled,
                    slots_response.slots,
                    model,
                ))
            },
            Err(err) => Ok(StatusUpdate::new(
                self.name.to_owned(),
                Some(err.to_string()),
                self.external_llamacpp_addr.to_owned(),
                None,
                None,
                vec![],
                None,
            )),
        }
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
        _listeners_per_fd: usize,
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
                                error!("Failed to report status: {err}");
                            }
                        }
                        Err(err) => {
                            error!("Failed to fetch status: {err}");
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