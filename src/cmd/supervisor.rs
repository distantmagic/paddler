use actix_web::web::Bytes;
use pingora::server::{configuration::Opt, Server};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast::channel;

use crate::errors::result::Result;
use crate::llamacpp::llamacpp_client::LlamacppClient;
use crate::supervisor::applying_service::ApplyingService;
use crate::supervisor::managing_service::ManagingService;

pub fn handle(
    local_llamacpp_addr: SocketAddr,
    llama_server_path: String,
    llamacpp_api_key: Option<String>,
    supervisor_management_addr: SocketAddr,
    monitoring_interval: Duration,
    _name: Option<String>,
) -> Result<()> {
    let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

    let llamacpp_client = LlamacppClient::new(local_llamacpp_addr, llamacpp_api_key.to_owned())?;

    let applying_service = ApplyingService::new(
        llamacpp_client,
        llama_server_path,
        monitoring_interval,
        // status_update_tx.clone(),
    )?;

    let manager_service = ManagingService::new(supervisor_management_addr)?;

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
