use std::net::SocketAddr;

use actix_web::web::Bytes;
use anyhow::Result;
use tokio::sync::broadcast;
use tokio::sync::oneshot;

use crate::agent::reporting_service::ReportingService;
use crate::service_manager::ServiceManager;

pub async fn handle(
    management_addr: SocketAddr,
    name: Option<String>,
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let (status_update_tx, _status_update_rx) = broadcast::channel::<Bytes>(1);
    let mut service_manager = ServiceManager::new();

    service_manager.add_service(ReportingService::new(management_addr, status_update_tx)?);

    service_manager.run_forever(shutdown_rx).await
}
