use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::oneshot;

use crate::service_manager::ServiceManager;
use crate::supervisor::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::supervisor::llamacpp_arbiter_service::LlamaCppArbiterService;
use crate::supervisor::management_socket_client_service::ManagementSocketClientService;
use crate::supervisor::reconciliation_queue::ReconciliationQueue;
use crate::supervisor::reconciliation_service::ReconciliationService;

pub async fn handle(
    llamacpp_listen_addr: SocketAddr,
    management_addr: SocketAddr,
    name: Option<String>,
) -> Result<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
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

    service_manager.run_forever(shutdown_rx).await
}
