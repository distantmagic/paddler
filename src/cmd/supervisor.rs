use pingora::server::{configuration::Opt, Server};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast::channel;

use crate::errors::result::Result;
use crate::supervisor::applying_service::ApplyingService;
use crate::supervisor::managing_service::ManagingService;

pub fn handle(
    local_llamacpp_addr: SocketAddr,
    llama_server_path: String,
    default_llamacpp_model: String,
    supervisor_management_addr: SocketAddr,
    monitoring_interval: Duration,
    _name: Option<String>,
) -> Result<()> {
    let (status_update_tx, status_update_rx) = channel::<String>(1);

    let applying_service = ApplyingService::new(
        local_llamacpp_addr,
        llama_server_path,
        default_llamacpp_model,
        monitoring_interval,
        status_update_rx,
    )?;

    let manager_service = ManagingService::new(supervisor_management_addr, status_update_tx)?;

    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();
    pingora_server.add_service(manager_service);
    pingora_server.add_service(applying_service);
    pingora_server.run_forever();
}
