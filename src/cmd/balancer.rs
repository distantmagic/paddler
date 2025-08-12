use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use tokio::sync::broadcast;
use tokio::sync::oneshot;

use super::handler::Handler;
use super::parse_duration;
use super::parse_socket_addr;
use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::balancer::chat_template_override_sender_collection::ChatTemplateOverrideSenderCollection;
use crate::balancer::embedding_sender_collection::EmbeddingSenderCollection;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::inference_service::InferenceService;
use crate::balancer::inference_service::configuration::Configuration as InferenceServiceConfiguration;
use crate::balancer::management_service::ManagementService;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
use crate::balancer::model_metadata_sender_collection::ModelMetadataSenderCollection;
use crate::balancer::reconciliation_service::ReconciliationService;
use crate::balancer::state_database::File;
use crate::balancer::state_database::Memory;
use crate::balancer::state_database::StateDatabase;
use crate::balancer::state_database_type::StateDatabaseType;
use crate::balancer::statsd_service::StatsdService;
use crate::balancer::statsd_service::configuration::Configuration as StatsdServiceConfiguration;
#[cfg(feature = "web_admin_panel")]
use crate::balancer::web_admin_panel_service::WebAdminPanelService;
#[cfg(feature = "web_admin_panel")]
use crate::balancer::web_admin_panel_service::configuration::Configuration as WebAdminPanelServiceConfiguration;
#[cfg(feature = "web_admin_panel")]
use crate::balancer::web_admin_panel_service::template_data::TemplateData;
use crate::balancer_applicable_state_holder::BalancerApplicableStateHolder;
use crate::service_manager::ServiceManager;

#[derive(Parser)]
pub struct Balancer {
    #[arg(long, default_value = "10000", value_parser = parse_duration)]
    /// Specifies how long a request can stay in the buffer before it is processed.
    /// If the request stays in the buffer longer than this time, it is rejected with the 504 error
    buffered_request_timeout: Duration,

    #[cfg(feature = "compat_openai")]
    #[arg(long, default_value = "127.0.0.1:8070", value_parser = parse_socket_addr)]
    /// Address of the OpenAI-compatible API server (enabled only if this address is specified)
    compat_openai_addr: Option<SocketAddr>,

    #[arg(long, default_value = "127.0.0.1:8061", value_parser = parse_socket_addr)]
    /// Address of the inference server
    inference_addr: SocketAddr,

    #[arg(long, default_value = "5000", value_parser = parse_duration)]
    /// The timeout (in milliseconds) for generating a single token or a single embedding
    inference_item_timeout: Duration,

    #[arg(
        long = "inference-cors-allowed-host",
        action = clap::ArgAction::Append
    )]
    /// Allowed CORS host for the inference service (can be specified multiple times)
    inference_cors_allowed_hosts: Vec<String>,

    #[arg(long, default_value = "127.0.0.1:8060", value_parser = parse_socket_addr)]
    /// This is where you can manage your Paddler setup and the agents connect to
    management_addr: SocketAddr,

    #[arg(
        long = "management-cors-allowed-host",
        action = clap::ArgAction::Append
    )]
    /// Allowed CORS host for the management service (can be specified multiple times)
    management_cors_allowed_hosts: Vec<String>,

    #[arg(long, default_value = "30")]
    /// The maximum number of buffered requests.
    /// If the buffer is full then new requests are rejected with the 503 error
    max_buffered_requests: i32,

    #[arg(long, default_value = "memory://")]
    /// Balancer state database URL. Supported: memory, memory://, or file:///path (optional)
    state_database: StateDatabaseType,

    #[arg(long, value_parser = parse_socket_addr)]
    /// Address for the statsd server to report metrics to (enabled only if this address is specified)
    statsd_addr: Option<SocketAddr>,

    #[arg(long, default_value = "paddler_")]
    /// Prefix for statsd metrics
    statsd_prefix: String,

    #[arg(long, default_value = "10000", value_parser = parse_duration)]
    /// Interval (in milliseconds) at which the balancer will report metrics to statsd
    statsd_reporting_interval: Duration,

    #[arg(long, default_value = None, value_parser = parse_socket_addr)]
    /// Address of the web admin panel (enabled only if this address is specified)
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
                template_data: TemplateData {
                    buffered_request_timeout: self.buffered_request_timeout,
                    max_buffered_requests: self.max_buffered_requests,
                    management_addr: self.management_addr,
                    inference_addr: self.inference_addr,
                    statsd_addr: self.statsd_addr,
                    statsd_prefix: self.statsd_prefix.clone(),
                    statsd_reporting_interval: self.statsd_reporting_interval,
                },
            })
    }
}

#[async_trait]
impl Handler for Balancer {
    async fn handle(&self, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        let (balancer_desired_state_tx, balancer_desired_state_rx) = broadcast::channel(100);

        let agent_controller_pool = Arc::new(AgentControllerPool::new());
        let balancer_applicable_state_holder = Arc::new(BalancerApplicableStateHolder::new());
        let buffered_request_manager = Arc::new(BufferedRequestManager::new(
            agent_controller_pool.clone(),
            self.buffered_request_timeout,
            self.max_buffered_requests,
        ));
        let chat_template_override_sender_collection =
            Arc::new(ChatTemplateOverrideSenderCollection::new());
        let embedding_sender_collection = Arc::new(EmbeddingSenderCollection::new());
        let generate_tokens_sender_collection = Arc::new(GenerateTokensSenderCollection::new());
        let model_metadata_sender_collection = Arc::new(ModelMetadataSenderCollection::new());
        let mut service_manager = ServiceManager::new();
        let state_database: Arc<dyn StateDatabase> = match &self.state_database {
            StateDatabaseType::File(path) => Arc::new(File::new(
                balancer_desired_state_tx.clone(),
                path.to_owned(),
            )),
            StateDatabaseType::Memory => Arc::new(Memory::new(balancer_desired_state_tx.clone())),
        };

        service_manager.add_service(InferenceService {
            balancer_applicable_state_holder: balancer_applicable_state_holder.clone(),
            buffered_request_manager: buffered_request_manager.clone(),
            configuration: InferenceServiceConfiguration {
                addr: self.inference_addr,
                cors_allowed_hosts: self.inference_cors_allowed_hosts.clone(),
                inference_item_timeout: self.inference_item_timeout,
            },
            #[cfg(feature = "web_admin_panel")]
            web_admin_panel_service_configuration: self.get_web_admin_panel_service_configuration(),
        });

        service_manager.add_service(ManagementService {
            agent_controller_pool: agent_controller_pool.clone(),
            balancer_applicable_state_holder: balancer_applicable_state_holder.clone(),
            buffered_request_manager: buffered_request_manager.clone(),
            chat_template_override_sender_collection,
            configuration: self.get_management_service_configuration(),
            embedding_sender_collection,
            generate_tokens_sender_collection,
            model_metadata_sender_collection,
            state_database: state_database.clone(),
            statsd_prefix: self.statsd_prefix.clone(),
            #[cfg(feature = "web_admin_panel")]
            web_admin_panel_service_configuration: self.get_web_admin_panel_service_configuration(),
        });

        service_manager.add_service(ReconciliationService {
            agent_controller_pool: agent_controller_pool.clone(),
            balancer_applicable_state_holder,
            balancer_desired_state: state_database.read_balancer_desired_state().await?,
            balancer_desired_state_rx,
            is_converted_to_applicable_state: false,
        });

        if let Some(statsd_addr) = self.statsd_addr {
            service_manager.add_service(StatsdService {
                agent_controller_pool,
                buffered_request_manager,
                configuration: StatsdServiceConfiguration {
                    statsd_addr,
                    statsd_prefix: self.statsd_prefix.clone(),
                    statsd_reporting_interval: self.statsd_reporting_interval,
                },
            });
        }

        #[cfg(feature = "web_admin_panel")]
        if let Some(configuration) = self.get_web_admin_panel_service_configuration() {
            service_manager.add_service(WebAdminPanelService { configuration });
        }

        service_manager.run_forever(shutdown_rx).await
    }
}
