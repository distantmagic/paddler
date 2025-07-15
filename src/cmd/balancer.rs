use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(feature = "statsd_reporter")]
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use tokio::sync::oneshot;

use super::handler::Handler;
use super::parse_duration;
use super::parse_socket_addr;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::inference_service::configuration::Configuration as InferenceServiceConfiguration;
use crate::balancer::inference_service::InferenceService;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
use crate::balancer::management_service::ManagementService;
use crate::balancer::state_database::File;
use crate::balancer::state_database::Memory;
use crate::balancer::state_database::StateDatabase;
#[cfg(feature = "statsd_reporter")]
use crate::balancer::statsd_service::configuration::Configuration as StatsdServiceConfiguration;
#[cfg(feature = "statsd_reporter")]
use crate::balancer::statsd_service::StatsdService;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::WebDashboardService;
use crate::database_type::DatabaseType;
use crate::service_manager::ServiceManager;

#[derive(Parser)]
pub struct Balancer {
    #[arg(long, default_value = "10000", value_parser = parse_duration)]
    /// The request timeout (in milliseconds). For all requests that a timely response from an
    /// upstream isn't received for, the 504 (Gateway Timeout) error is issued.
    buffered_request_timeout: Duration,

    #[arg(long, default_value = "127.0.0.1:8061", value_parser = parse_socket_addr)]
    /// Address of the inference server
    inference_addr: SocketAddr,

    #[arg(
        long = "inference-cors-allowed-host",
        action = clap::ArgAction::Append
    )]
    /// Allowed CORS host (can be specified multiple times)
    inference_cors_allowed_hosts: Vec<String>,

    #[arg(long, default_value = "127.0.0.1:8060", value_parser = parse_socket_addr)]
    /// Address of the management server that the balancer will report to
    management_addr: SocketAddr,

    #[arg(
        long = "management-cors-allowed-host",
        action = clap::ArgAction::Append
    )]
    /// Allowed CORS host (can be specified multiple times)
    management_cors_allowed_hosts: Vec<String>,

    #[arg(long, default_value = "30")]
    /// The maximum number of buffered requests. Like with usual requests, the request timeout
    /// is also applied to buffered ones. If the maximum number is reached, all new requests are
    /// rejected with the 429 (Too Many Requests) error.
    max_buffered_requests: usize,

    #[arg(long, default_value = "memory://")]
    /// Balancer state database URL. Supported: memory, memory://, or file:///path (optional)
    state_database: DatabaseType,

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

    #[arg(long, default_value = None, value_parser = parse_socket_addr)]
    /// Address of the web management dashboard (if enabled)
    web_dashboard_addr: Option<SocketAddr>,
}

impl Balancer {
    fn get_mangement_service_configuration(&self) -> ManagementServiceConfiguration {
        ManagementServiceConfiguration {
            addr: self.management_addr,
            cors_allowed_hosts: self.management_cors_allowed_hosts.clone(),
        }
    }

    #[cfg(feature = "web_dashboard")]
    fn get_web_dashboard_service_configuration(&self) -> Option<WebDashboardServiceConfiguration> {
        self.web_dashboard_addr
            .map(|web_dashboard_addr| WebDashboardServiceConfiguration {
                addr: web_dashboard_addr,
                management_addr: self.management_addr,
            })
    }
}

#[async_trait]
impl Handler for Balancer {
    async fn handle(&self, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        let agent_controller_pool = Arc::new(AgentControllerPool::new());
        let mut service_manager = ServiceManager::new();
        let state_database: Arc<dyn StateDatabase> = match &self.state_database {
            DatabaseType::File(path) => Arc::new(File::new(path.to_owned())),
            DatabaseType::Memory => Arc::new(Memory::new()),
        };

        service_manager.add_service(InferenceService::new(
            agent_controller_pool.clone(),
            InferenceServiceConfiguration {
                addr: self.inference_addr,
                cors_allowed_hosts: self.inference_cors_allowed_hosts.clone(),
                // buffered_request_timeout: self.buffered_request_timeout,
                // max_buffered_requests: self.max_buffered_requests,
            },
            state_database.clone(),
            #[cfg(feature = "web_dashboard")]
            self.get_web_dashboard_service_configuration(),
        ));

        service_manager.add_service(ManagementService::new(
            agent_controller_pool.clone(),
            self.get_mangement_service_configuration(),
            state_database,
            #[cfg(feature = "web_dashboard")]
            self.get_web_dashboard_service_configuration(),
        ));

        #[cfg(feature = "statsd_reporter")]
        if let Some(statsd_addr) = self.statsd_addr {
            service_manager.add_service(StatsdService::new(
                agent_controller_pool.clone(),
                StatsdServiceConfiguration {
                    statsd_addr,
                    statsd_prefix: self.statsd_prefix.clone(),
                    statsd_reporting_interval: self.statsd_reporting_interval,
                },
            )?);
        }

        #[cfg(feature = "web_dashboard")]
        if let Some(web_dashboard_service_configuration) =
            self.get_web_dashboard_service_configuration()
        {
            service_manager.add_service(WebDashboardService::new(
                agent_controller_pool,
                web_dashboard_service_configuration,
            ));
        }

        service_manager.run_forever(shutdown_rx).await
    }
}
