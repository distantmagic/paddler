use std::net::SocketAddr;
use std::time::Duration;

use actix_web::web::Bytes;
use anyhow::Result;
use pingora::server::configuration::Opt;
use pingora::server::Server;
use tokio::sync::broadcast::channel;

use crate::agent::monitoring_service::MonitoringService;
use crate::agent::reporting_service::ReportingService;
use crate::llamacpp::llamacpp_client::LlamacppClient;

pub fn handle(
    external_llamacpp_addr: SocketAddr,
    local_llamacpp_addr: SocketAddr,
    llamacpp_api_key: Option<String>,
    management_addr: SocketAddr,
    monitoring_interval: Duration,
    name: Option<String>,
    check_model: bool, // Include the check_model flag
) -> Result<()> {
    let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

    let llamacpp_client = LlamacppClient::new(local_llamacpp_addr, llamacpp_api_key)?;

    let monitoring_service = MonitoringService::new(
        external_llamacpp_addr,
        llamacpp_client,
        monitoring_interval,
        name,
        status_update_tx.clone(),
        check_model, // Pass the check_model flag
    )?;

    let reporting_service = ReportingService::new(management_addr, status_update_tx)?;

    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();
    pingora_server.add_service(monitoring_service);
    pingora_server.add_service(reporting_service);
    pingora_server.run_forever();
}