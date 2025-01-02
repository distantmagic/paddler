use pingora::server::{configuration::Opt, Server};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast::{channel};

use crate::errors::result::Result;
use crate::supervisor::applying_service::ApplyingService;
use crate::supervisor::llamacpp_configuration::LlamacppConfiguration;
use crate::supervisor::managing_service::ManagingService;

pub fn handle(
    addr: SocketAddr,
    llama_path: String,
    model_path: String,
    threads_number: i8,
    supervisor_management_addr: SocketAddr,
    monitoring_interval: Duration,
    _name: Option<String>,
) -> Result<()> {
    let (update_llamacpp_tx, update_llamacpp_rx) = channel::<String>(1);

    let manager_service = ManagingService::new(supervisor_management_addr, update_llamacpp_tx)?;

    let llamacpp_options = LlamacppConfiguration::new(
        addr,
        llama_path,
        model_path,
        threads_number
    );
    
    let applying_service = ApplyingService::new(
        llamacpp_options,
        monitoring_interval,
        update_llamacpp_rx,
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
