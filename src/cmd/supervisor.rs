use pingora::server::{configuration::Opt, Server};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast::{channel, Sender};

use crate::errors::result::Result;
use crate::supervisor::applying_service::ApplyingService;
use crate::supervisor::managing_service::ManagingService;

#[derive(Clone)]
pub struct UpdateLlamacpp {
    pub update_binary_tx: Sender<String>,
    pub update_model_tx: Sender<String>,
    pub update_addr: Sender<String>,
}

pub fn handle(
    local_llamacpp_addr: SocketAddr,
    llama_server_path: String,
    default_llamacpp_model: String,
    supervisor_management_addr: SocketAddr,
    monitoring_interval: Duration,
    _name: Option<String>,
) -> Result<()> {
    let (update_binary_tx, update_binary_rx) = channel::<String>(1);
    let (update_model_tx, update_model_rx) = channel::<String>(1);
    let (update_addr, update_addr_rx) = channel::<String>(1);

    let update_channels = UpdateLlamacpp {
        update_binary_tx,
        update_model_tx,
        update_addr,
    };

    let manager_service = ManagingService::new(supervisor_management_addr, update_channels)?;

    let applying_service = ApplyingService::new(
        local_llamacpp_addr,
        llama_server_path,
        default_llamacpp_model,
        monitoring_interval,
        update_model_rx,
        update_binary_rx,
        update_addr_rx,
    )?;

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
