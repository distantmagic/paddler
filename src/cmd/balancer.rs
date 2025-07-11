use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(feature = "statsd_reporter")]
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tokio::sync::oneshot;

use super::parse_duration;
use super::parse_socket_addr;
use crate::balancer::fleet_management_database::File;
use crate::balancer::fleet_management_database::Memory;
use crate::balancer::fleet_management_database_type::FleetManagementDatabaseType;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
use crate::balancer::management_service::ManagementService;
#[cfg(feature = "statsd_reporter")]
use crate::balancer::statsd_service::configuration::Configuration as StatsdServiceConfiguration;
#[cfg(feature = "statsd_reporter")]
use crate::balancer::statsd_service::StatsdService;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;
// use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::WebDashboardService;
use crate::service_manager::ServiceManager;

#[derive(Parser)]
pub struct Balancer {
    #[arg(long, default_value = "10000", value_parser = parse_duration)]
    /// The request timeout (in milliseconds). For all requests that a timely response from an
    /// upstream isn't received for, the 504 (Gateway Timeout) error is issued.
    buffered_request_timeout: Duration,

    #[arg(long, default_value = "memory://")]
    // Fleet management database URL. Supported: memory, memory://, or file:///path (optional)
    fleet_management_database: FleetManagementDatabaseType,

    #[arg(long)]
    /// Enable registering agent-managed llama.cpp instances in the balancer
    fleet_management_enable: bool,

    #[arg(long, default_value = "127.0.0.1:8060", value_parser = parse_socket_addr)]
    /// Address of the management server that the balancer will report to
    management_addr: SocketAddr,

    #[arg(
        long = "management-cors-allowed-host",
        help = "Allowed CORS host (can be specified multiple times)",
        action = clap::ArgAction::Append
    )]
    management_cors_allowed_hosts: Vec<String>,

    #[arg(long, default_value = "30")]
    /// The maximum number of buffered requests. Like with usual requests, the request timeout
    /// is also applied to buffered ones. If the maximum number is reached, all new requests are
    /// rejected with the 429 (Too Many Requests) error.
    max_buffered_requests: usize,

    #[arg(long, default_value = "127.0.0.1:8061", value_parser = parse_socket_addr)]
    /// Address of the reverse proxy server
    reverseproxy_addr: SocketAddr,

    #[arg(long)]
    /// Enable the web metrics endpoint
    metrics_endpoint_enable: bool,

    #[cfg(feature = "statsd_reporter")]
    #[arg(long, value_parser = parse_socket_addr)]
    /// Address of the statsd server to report metrics to
    statsd_addr: Option<SocketAddr>,

    #[cfg(feature = "statsd_reporter")]
    #[arg(long, default_value = "paddler")]
    /// Prefix for statsd metrics
    statsd_prefix: String,

    #[cfg(feature = "statsd_reporter")]
    #[arg(long, default_value = "10000", value_parser = parse_duration)]
    /// Interval (in milliseconds) at which the balancer will report metrics to statsd
    statsd_reporting_interval: Duration,

    #[arg(long, default_value = "127.0.0.1:8061", value_parser = parse_socket_addr)]
    /// Address of the web management dashboard (if enabled)
    web_dashboard_addr: Option<SocketAddr>,

    #[cfg(feature = "web_dashboard")]
    #[arg(long, default_value = "false")]
    /// Enable the web management dashboard
    web_dashboard_enable: bool,
}

impl Balancer {
    pub async fn handle(&self, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        let mut service_manager = ServiceManager::new();
        // let upstream_peer_pool = Arc::new(UpstreamPeerPool::new());

        service_manager.add_service(ManagementService::new(
            self.get_mangement_service_configuration(),
            match &self.fleet_management_database {
                FleetManagementDatabaseType::File(path) => Arc::new(File::new(path.to_owned())),
                FleetManagementDatabaseType::Memory => Arc::new(Memory::new()),
            },
            // upstream_peer_pool.clone(),
            #[cfg(feature = "web_dashboard")]
            self.get_web_dashboard_service_configuration(),
        ));

        #[cfg(feature = "statsd_reporter")]
        if let Some(statsd_addr) = self.statsd_addr {
            service_manager.add_service(StatsdService::new(
                StatsdServiceConfiguration {
                    statsd_addr,
                    statsd_prefix: self.statsd_prefix.clone(),
                    statsd_reporting_interval: self.statsd_reporting_interval,
                },
                // upstream_peer_pool.clone(),
            )?);
        }

        #[cfg(feature = "web_dashboard")]
        if let Some(web_dashboard_service_configuration) =
            self.get_web_dashboard_service_configuration()
        {
            service_manager.add_service(WebDashboardService::new(
                web_dashboard_service_configuration,
                // upstream_peer_pool.clone(),
            ));
        }

        service_manager.run_forever(shutdown_rx).await
    }

    fn get_mangement_service_configuration(&self) -> ManagementServiceConfiguration {
        ManagementServiceConfiguration {
            addr: self.management_addr,
            cors_allowed_hosts: self.management_cors_allowed_hosts.clone(),
            fleet_management_enable: self.fleet_management_enable,
            metrics_endpoint_enable: self.metrics_endpoint_enable,
        }
    }

    #[cfg(feature = "web_dashboard")]
    fn get_web_dashboard_service_configuration(&self) -> Option<WebDashboardServiceConfiguration> {
        if self.web_dashboard_enable {
            self.web_dashboard_addr
                .map(|web_dashboard_addr| WebDashboardServiceConfiguration {
                    addr: web_dashboard_addr,
                    management_addr: self.management_addr,
                })
        } else {
            None
        }
    }
}
