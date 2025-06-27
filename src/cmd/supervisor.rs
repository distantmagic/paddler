use std::net::SocketAddr;
use std::sync::Arc;

use pingora::server::configuration::Opt;
use pingora::server::Server;

use crate::errors::result::Result;
use crate::supervisor::api_service::ApiService;
use crate::supervisor::reconciliation_queue::ReconciliationQueue;
use crate::supervisor::reconciliation_service::ReconciliationService;

pub fn handle(
    api_addr: SocketAddr,
    management_addr: SocketAddr,
    name: Option<String>,
) -> Result<()> {
    let api_service = ApiService::new(api_addr);
    let reconciliation_queue = Arc::new(ReconciliationQueue::new()?);
    let reconciliation_service = ReconciliationService::new(name, reconciliation_queue.clone())?;

    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();
    pingora_server.add_service(api_service);
    pingora_server.add_service(reconciliation_service);
    pingora_server.run_forever();
}
