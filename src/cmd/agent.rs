use std::net::SocketAddr;
use std::sync::Arc;

use actix_web::web::Bytes;
use anyhow::Result;
use tokio::sync::broadcast;
use tokio::sync::oneshot;

use crate::agent::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::agent::llamacpp_arbiter_service::LlamaCppArbiterService;
use crate::agent::management_socket_client_service::ManagementSocketClientService;
use crate::agent::reconciliation_queue::ReconciliationQueue;
use crate::agent::reconciliation_service::ReconciliationService;
use crate::agent::reporting_service::ReportingService;
use crate::service_manager::ServiceManager;

pub async fn handle(
    llamacpp_listen_addr: SocketAddr,
    management_addr: SocketAddr,
    name: Option<String>,
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let (status_update_tx, _status_update_rx) = broadcast::channel::<Bytes>(1);
    let llamacpp_applicable_state_holder = Arc::new(LlamaCppApplicableStateHolder::new());
    let reconciliation_queue = Arc::new(ReconciliationQueue::new()?);
    let mut service_manager = ServiceManager::new();

    service_manager.add_service(LlamaCppArbiterService::new(
        llamacpp_applicable_state_holder.clone(),
        llamacpp_listen_addr,
    )?);

    service_manager.add_service(ManagementSocketClientService::new(
        management_addr,
        name,
        reconciliation_queue.clone(),
    )?);

    service_manager.add_service(ReconciliationService::new(
        llamacpp_applicable_state_holder,
        reconciliation_queue,
    )?);

    service_manager.add_service(ReportingService::new(management_addr, status_update_tx)?);

    service_manager.run_forever(shutdown_rx).await
}
