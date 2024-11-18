use actix_web::web::Bytes;
use pingora::server::configuration::Opt;
use pingora::server::Server;
use std::net::SocketAddr;
use tokio::sync::broadcast::channel;
use url::Url;

use crate::agent::monitoring_service::MonitoringService;
use crate::agent::reporting_service::ReportingService;
use crate::errors::result::Result;
use crate::llamacpp::llamacpp_client::LlamacppClient;

pub fn handle(
    external_llamacpp_addr: SocketAddr,
    local_llamacpp_addr: Url,
    local_llamacpp_api_key: Option<String>,
    management_addr: Url,
    name: Option<String>,
) -> Result<()> {
    let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

    let llamacpp_client = LlamacppClient::new(local_llamacpp_addr, local_llamacpp_api_key)?;

    let monitoring_service = MonitoringService::new(
        external_llamacpp_addr,
        llamacpp_client,
        name,
        status_update_tx.clone(),
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
