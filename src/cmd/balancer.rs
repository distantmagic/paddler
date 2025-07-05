use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(feature = "statsd_reporter")]
use std::time::Duration;

use anyhow::Result;
use pingora::proxy::http_proxy_service;
use pingora::server::configuration::Opt;
use pingora::server::Server;

#[cfg(feature = "supervisor")]
use crate::balancer::fleet_management_database::FleetManagementDatabase;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
use crate::balancer::management_service::ManagementService;
use crate::balancer::proxy_service::ProxyService;
#[cfg(feature = "statsd_reporter")]
use crate::balancer::statsd_service::configuration::Configuration as StatsdServiceConfiguration;
#[cfg(feature = "statsd_reporter")]
use crate::balancer::statsd_service::StatsdService;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::WebDashboardService;

#[allow(clippy::too_many_arguments)]
pub fn handle(
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
    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();

    let upstream_peer_pool = Arc::new(UpstreamPeerPool::new());

    let mut proxy_service = http_proxy_service(
        &pingora_server.configuration,
        ProxyService::new(
            rewrite_host_header,
            slots_endpoint_enable,
            upstream_peer_pool.clone(),
            buffered_request_timeout,
            max_buffered_requests,
        ),
    );

    proxy_service.add_tcp(&reverseproxy_addr.clone().to_string());

    pingora_server.add_service(proxy_service);
    pingora_server.add_service(ManagementService::new(
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

        pingora_server.add_service(statsd_service);
    }

    #[cfg(feature = "web_dashboard")]
    if let Some(web_dashboard_service_configuration) = web_dashboard_service_configuration {
        let web_dashboard_service = WebDashboardService::new(
            web_dashboard_service_configuration,
            upstream_peer_pool.clone(),
        );

        pingora_server.add_service(web_dashboard_service);
    }

    pingora_server.run_forever();
}
