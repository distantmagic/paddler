use pingora::server::{configuration::Opt, Server};
use std::net::SocketAddr;
use tokio::sync::broadcast::channel;

use crate::errors::result::Result;
use crate::supervisor::application_service::ApplyingService;
use crate::supervisor::management_service::ManagingService;

pub fn handle(args: Vec<String>, supervisor_addr: SocketAddr) -> Result<()> {
    let (update_llamacpp_tx, update_llamacpp_rx) = channel::<Vec<String>>(1);

    let manager_service = ManagingService::new(supervisor_addr, update_llamacpp_tx)?;

    let applying_service =
        ApplyingService::new(args, update_llamacpp_rx, supervisor_addr)?;

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
