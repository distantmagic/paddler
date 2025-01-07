use pingora::server::{configuration::Opt, Server};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast::channel;

use crate::errors::result::Result;
use crate::supervisor::applying_service::ApplyingService;
use crate::supervisor::managing_service::ManagingService;

pub fn handle(
    args: Vec<String>,
    supervisor_addr: SocketAddr,
    monitoring_interval: Duration,
) -> Result<()> {
    let (update_llamacpp_tx, update_llamacpp_rx) = channel::<Vec<String>>(1);

    let manager_service = ManagingService::new(supervisor_addr, update_llamacpp_tx)?;

    let applying_service = ApplyingService::new(args, monitoring_interval, update_llamacpp_rx)?;

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
