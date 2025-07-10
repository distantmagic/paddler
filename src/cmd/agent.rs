use std::net::SocketAddr;
use std::time::Duration;

use actix_web::web::Bytes;
use anyhow::Result;
use tokio::sync::broadcast;
use tokio::sync::oneshot;

use crate::agent::monitoring_service::MonitoringService;
use crate::agent::reporting_service::ReportingService;
use crate::llamacpp::llamacpp_client::LlamacppClient;
use crate::service_manager::ServiceManager;

pub async fn handle(
    external_llamacpp_addr: SocketAddr,
    local_llamacpp_addr: SocketAddr,
    llamacpp_api_key: Option<String>,
    management_addr: SocketAddr,
    monitoring_interval: Duration,
    name: Option<String>,
) -> Result<()> {
    let (status_update_tx, _status_update_rx) = broadcast::channel::<Bytes>(1);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let llamacpp_client = LlamacppClient::new(local_llamacpp_addr, llamacpp_api_key)?;
    let mut service_manager = ServiceManager::new();

    service_manager.add_service(MonitoringService::new(
        external_llamacpp_addr,
        llamacpp_client,
        monitoring_interval,
        name,
        status_update_tx.clone(),
    )?);
    service_manager.add_service(ReportingService::new(management_addr, status_update_tx)?);

    service_manager.run_forever(shutdown_rx).await
}
