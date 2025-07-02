use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use pingora::server::configuration::Opt;
use pingora::server::Server;

use crate::supervisor::management_socket_client_service::ManagementSocketClientService;
use crate::supervisor::reconciliation_queue::ReconciliationQueue;
use crate::supervisor::reconciliation_service::ReconciliationService;

pub fn handle(
    llamacpp_listen_addr: SocketAddr,
    management_addr: SocketAddr,
    name: Option<String>,
) -> Result<()> {
    let reconciliation_queue = Arc::new(ReconciliationQueue::new()?);
    let management_socket_client_service =
        ManagementSocketClientService::new(management_addr, name, reconciliation_queue.clone())?;
    let reconciliation_service = ReconciliationService::new(reconciliation_queue)?;

    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();
    pingora_server.add_service(management_socket_client_service);
    pingora_server.add_service(reconciliation_service);
    pingora_server.run_forever();
}
