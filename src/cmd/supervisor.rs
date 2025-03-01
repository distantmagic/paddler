#[cfg(not(feature = "etcd"))]
use color_eyre::owo_colors::OwoColorize;
use pingora::server::{configuration::Opt, Server};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::sync::broadcast::channel;

use crate::errors::result::Result;
use crate::supervisor::application_service::ApplicationService;
use crate::supervisor::configuration_service::ConfigurationService;
use crate::supervisor::management_service::ManagementService;

pub fn handle(
    binary: String,
    model: String,
    port: u16,
    supervisor_addr: SocketAddr,
    #[cfg(feature = "etcd")] etcd: Option<SocketAddr>,
    file_path: Option<PathBuf>,
) -> Result<()> {
    #[cfg(not(feature = "etcd"))]
    if file_path == None {
        panic!(
            "{} {} {}",
            "error:".red(),
            "the following required arguments were not provided:",
            "--etcd <ETCD>".green()
        );
    }

    let (update_llamacpp_tx, update_llamacpp_rx) = channel::<Vec<String>>(1);
    let (update_config_tx, update_config_rx) = channel::<Vec<String>>(1);

    let management_service = ManagementService::new(supervisor_addr, update_llamacpp_tx)?;

    let application_service = ApplicationService::new(
        binary,
        model,
        port,
        #[cfg(feature = "etcd")]
        etcd,
        file_path.clone(),
        update_llamacpp_rx,
        update_config_tx,
    )?;

    let configuration_service = ConfigurationService::new(
        update_config_rx,
        #[cfg(feature = "etcd")]
        etcd,
        file_path,
    )?;

    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();
    pingora_server.add_service(management_service);
    pingora_server.add_service(application_service);
    pingora_server.add_service(configuration_service);
    pingora_server.run_forever();
}
