use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(feature = "statsd_reporter")]
use std::time::Duration;

use anyhow::Result;
use tokio::sync::oneshot;

use crate::balancer::fleet_management_database::FleetManagementDatabase;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
use crate::balancer::management_service::ManagementService;
#[cfg(feature = "statsd_reporter")]
use crate::balancer::statsd_service::configuration::Configuration as StatsdServiceConfiguration;
#[cfg(feature = "statsd_reporter")]
use crate::balancer::statsd_service::StatsdService;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::WebDashboardService;
use crate::service_manager::ServiceManager;

#[allow(clippy::too_many_arguments)]
pub async fn handle(
    buffered_request_timeout: Duration,
    fleet_management_database: Arc<dyn FleetManagementDatabase>,
    management_service_configuration: ManagementServiceConfiguration,
    max_buffered_requests: usize,
    reverseproxy_addr: SocketAddr,
    rewrite_host_header: bool,
    slots_endpoint_enable: bool,
    #[cfg(feature = "statsd_reporter")] statsd_service_configuration_maybe: Option<
        StatsdServiceConfiguration,
    >,
    #[cfg(feature = "web_dashboard")] web_dashboard_service_configuration: Option<
        WebDashboardServiceConfiguration,
    >,
) -> Result<()> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let mut service_manager = ServiceManager::new();
    let upstream_peer_pool = Arc::new(UpstreamPeerPool::new());

    service_manager.add_service(ManagementService::new(
        management_service_configuration,
        fleet_management_database,
        upstream_peer_pool.clone(),
        #[cfg(feature = "web_dashboard")]
        web_dashboard_service_configuration.clone(),
    ));

    #[cfg(feature = "statsd_reporter")]
    if let Some(statsd_service_configuration) = statsd_service_configuration_maybe {
        let statsd_service =
            StatsdService::new(statsd_service_configuration, upstream_peer_pool.clone())?;

        service_manager.add_service(statsd_service);
    }

    #[cfg(feature = "web_dashboard")]
    if let Some(web_dashboard_service_configuration) = web_dashboard_service_configuration {
        let web_dashboard_service = WebDashboardService::new(
            web_dashboard_service_configuration,
            upstream_peer_pool.clone(),
        );

        service_manager.add_service(web_dashboard_service);
    }

    service_manager.run_forever(shutdown_rx).await
}
