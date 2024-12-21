use actix_web::web::Bytes;
use pingora::server::{configuration::Opt, Server};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::broadcast::channel;

use crate::errors::result::Result;
use crate::llamacpp::llamacpp_client::LlamacppClient;
use crate::supervisor::applying_service::ApplyingService;
use crate::supervisor::supervising_service::SupervisingService;

pub fn handle(
    local_llamacpp_addr: SocketAddr,
    llama_server_path: String,
    llamacpp_api_key: Option<String>,
    management_addr: SocketAddr,
    monitoring_interval: Duration,
    name: Option<String>,
) -> Result<()> {
    let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

    let llamacpp_client = LlamacppClient::new(local_llamacpp_addr, llamacpp_api_key.to_owned())?;

    let supervising_service = SupervisingService::new(
        monitoring_interval,
        llamacpp_client,
        name,
        local_llamacpp_addr,
        status_update_tx.clone(),
    )?;

    let applying_service =
        ApplyingService::new(management_addr, llama_server_path, status_update_tx)?;

    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();
    pingora_server.add_service(supervising_service);
    pingora_server.add_service(applying_service);
    pingora_server.run_forever();
}
