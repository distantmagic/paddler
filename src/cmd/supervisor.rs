use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use pingora::server::configuration::Opt;
use pingora::server::Server;

use crate::supervisor::llamacpp_applicable_state_holder::LlamaCppApplicableStateHolder;
use crate::supervisor::llamacpp_arbiter_service::LlamaCppArbiterService;
use crate::supervisor::management_socket_client_service::ManagementSocketClientService;
use crate::supervisor::reconciliation_queue::ReconciliationQueue;
use crate::supervisor::reconciliation_service::ReconciliationService;

pub fn handle(
    llamacpp_listen_addr: SocketAddr,
    management_addr: SocketAddr,
    name: Option<String>,
) -> Result<()> {
    let llamacpp_applicable_state_holder = Arc::new(LlamaCppApplicableStateHolder::new());
    let reconciliation_queue = Arc::new(ReconciliationQueue::new()?);

    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();
    pingora_server.add_service(LlamaCppArbiterService::new(
        llamacpp_applicable_state_holder.clone(),
        llamacpp_listen_addr,
    )?);
    pingora_server.add_service(ManagementSocketClientService::new(
        management_addr,
        name,
        reconciliation_queue.clone(),
    )?);
    pingora_server.add_service(ReconciliationService::new(
        llamacpp_applicable_state_holder,
        reconciliation_queue,
    )?);
    pingora_server.run_forever();
}
