use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use tokio::sync::oneshot;

use super::handler::Handler;
use super::parse_duration;
use super::parse_socket_addr;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::inference_service::configuration::Configuration as InferenceServiceConfiguration;
use crate::balancer::inference_service::InferenceService;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
use crate::balancer::management_service::ManagementService;
use crate::balancer::state_database::File;
use crate::balancer::state_database::Memory;
use crate::balancer::state_database::StateDatabase;
use crate::balancer::statsd_service::configuration::Configuration as StatsdServiceConfiguration;
use crate::balancer::statsd_service::StatsdService;
#[cfg(feature = "web_admin_panel")]
use crate::balancer::web_admin_panel_service::configuration::Configuration as WebAdminPanelServiceConfiguration;
#[cfg(feature = "web_admin_panel")]
use crate::balancer::web_admin_panel_service::WebAdminPanelService;
use crate::database_type::DatabaseType;
use crate::service_manager::ServiceManager;

#[derive(Parser)]
pub struct Balancer {
    #[arg(long, default_value = "10000", value_parser = parse_duration)]
    /// The request timeout (in milliseconds). For all requests that a timely response from an
    /// upstream isn't received for, the 504 (Gateway Timeout) error is issued.
    request_timeout: Duration,

    #[arg(long, default_value = "127.0.0.1:8061", value_parser = parse_socket_addr)]
    /// Address of the inference server.
    inference_addr: SocketAddr,

    #[arg(long, default_value = "10000", value_parser = parse_duration)]
    /// The timeout (in milliseconds) for generating a single token.
    inference_token_timeout: Duration,

    #[arg(
        long = "inference-cors-allowed-host",
        action = clap::ArgAction::Append
    )]
    /// Allowed CORS host (can be specified multiple times).
    inference_cors_allowed_hosts: Vec<String>,

    #[arg(long, default_value = "127.0.0.1:8060", value_parser = parse_socket_addr)]
    /// Address of the management server that the balancer will report to.
    management_addr: SocketAddr,

    #[arg(
        long = "management-cors-allowed-host",
        action = clap::ArgAction::Append
    )]
    /// Allowed CORS host (can be specified multiple times).
    management_cors_allowed_hosts: Vec<String>,

    #[arg(long, default_value = "30")]
    /// The maximum number of buffered requests. Like with usual requests, the request timeout
    /// is also applied to buffered ones. If the maximum number is reached, all new requests are
    /// rejected with the 429 (Too Many Requests) error.
    max_buffered_requests: i32,

    #[arg(long, default_value = "memory://")]
    /// Balancer state database URL. Supported: memory, memory://, or file:///path (optional).
    state_database: DatabaseType,

    #[arg(long, value_parser = parse_socket_addr)]
    /// Address of the statsd server to report metrics to.
    statsd_addr: Option<SocketAddr>,

    #[arg(long, default_value = "paddler")]
    /// Prefix for statsd metrics.
    statsd_prefix: String,

    #[arg(long, default_value = "10000", value_parser = parse_duration)]
    /// Interval (in milliseconds) at which the balancer will report metrics to statsd.
    statsd_reporting_interval: Duration,

    #[arg(long, default_value = None, value_parser = parse_socket_addr)]
    /// Address of the web management dashboard (if enabled).
    web_admin_panel_addr: Option<SocketAddr>,
}

impl Balancer {
    fn get_management_service_configuration(&self) -> ManagementServiceConfiguration {
        ManagementServiceConfiguration {
            addr: self.management_addr,
            cors_allowed_hosts: self.management_cors_allowed_hosts.clone(),
        }
    }

    #[cfg(feature = "web_admin_panel")]
    fn get_web_admin_panel_service_configuration(
        &self,
    ) -> Option<WebAdminPanelServiceConfiguration> {
        self.web_admin_panel_addr
            .map(|web_admin_panel_addr| WebAdminPanelServiceConfiguration {
                addr: web_admin_panel_addr,
                inference_addr: self.inference_addr,
                management_addr: self.management_addr,
            })
    }
}

#[async_trait]
impl Handler for Balancer {
    async fn handle(&self, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        let agent_controller_pool = Arc::new(AgentControllerPool::new());
        let buffered_request_manager = Arc::new(BufferedRequestManager::new(
            agent_controller_pool.clone(),
            self.request_timeout,
            self.max_buffered_requests,
        ));
        let generate_tokens_sender_collection = Arc::new(GenerateTokensSenderCollection::new());
        let mut service_manager = ServiceManager::new();
        let state_database: Arc<dyn StateDatabase> = match &self.state_database {
            DatabaseType::File(path) => Arc::new(File::new(path.to_owned())),
            DatabaseType::Memory => Arc::new(Memory::new()),
        };

        service_manager.add_service(InferenceService::new(
            agent_controller_pool.clone(),
            buffered_request_manager.clone(),
            InferenceServiceConfiguration {
                addr: self.inference_addr,
                cors_allowed_hosts: self.inference_cors_allowed_hosts.clone(),
                inference_token_timeout: self.inference_token_timeout,
            },
            state_database.clone(),
            #[cfg(feature = "web_admin_panel")]
            self.get_web_admin_panel_service_configuration(),
        ));

        service_manager.add_service(ManagementService::new(
            agent_controller_pool.clone(),
            buffered_request_manager.clone(),
            self.get_management_service_configuration(),
            generate_tokens_sender_collection.clone(),
            state_database,
            #[cfg(feature = "web_admin_panel")]
            self.get_web_admin_panel_service_configuration(),
        ));

        if let Some(statsd_addr) = self.statsd_addr {
            service_manager.add_service(StatsdService::new(
                agent_controller_pool.clone(),
                buffered_request_manager,
                StatsdServiceConfiguration {
                    statsd_addr,
                    statsd_prefix: self.statsd_prefix.clone(),
                    statsd_reporting_interval: self.statsd_reporting_interval,
                },
            )?);
        }

        #[cfg(feature = "web_admin_panel")]
        if let Some(web_admin_panel_service_configuration) =
            self.get_web_admin_panel_service_configuration()
        {
            service_manager.add_service(WebAdminPanelService::new(
                agent_controller_pool,
                web_admin_panel_service_configuration,
            ));
        }

        service_manager.run_forever(shutdown_rx).await
    }
}
