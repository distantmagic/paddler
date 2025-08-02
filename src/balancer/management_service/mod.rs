pub mod configuration;
pub mod http_route;

use std::sync::Arc;

use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::broadcast;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::buffered_request_manager::BufferedRequestManager;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::http_route as common_http_route;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
use crate::balancer::model_metadata_sender_collection::ModelMetadataSenderCollection;
use crate::balancer::state_database::StateDatabase;
#[cfg(feature = "web_admin_panel")]
use crate::balancer::web_admin_panel_service::configuration::Configuration as WebAdminPanelServiceConfiguration;
use crate::create_cors_middleware::create_cors_middleware;
use crate::service::Service;

pub struct ManagementService {
    agent_controller_pool: Arc<AgentControllerPool>,
    buffered_request_manager: Arc<BufferedRequestManager>,
    configuration: ManagementServiceConfiguration,
    generate_tokens_sender_collection: Arc<GenerateTokensSenderCollection>,
    model_metadata_sender_collection: Arc<ModelMetadataSenderCollection>,
    state_database: Arc<dyn StateDatabase>,
    #[cfg(feature = "web_admin_panel")]
    web_admin_panel_service_configuration: Option<WebAdminPanelServiceConfiguration>,
}

impl ManagementService {
    pub fn new(
        agent_controller_pool: Arc<AgentControllerPool>,
        buffered_request_manager: Arc<BufferedRequestManager>,
        configuration: ManagementServiceConfiguration,
        generate_tokens_sender_collection: Arc<GenerateTokensSenderCollection>,
        model_metadata_sender_collection: Arc<ModelMetadataSenderCollection>,
        state_database: Arc<dyn StateDatabase>,
        #[cfg(feature = "web_admin_panel")] web_admin_panel_service_configuration: Option<
            WebAdminPanelServiceConfiguration,
        >,
    ) -> Self {
        ManagementService {
            agent_controller_pool,
            buffered_request_manager,
            configuration,
            generate_tokens_sender_collection,
            model_metadata_sender_collection,
            state_database,
            #[cfg(feature = "web_admin_panel")]
            web_admin_panel_service_configuration,
        }
    }
}

#[async_trait]
impl Service for ManagementService {
    fn name(&self) -> &'static str {
        "balancer::management_service"
    }

    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        #[allow(unused_mut)]
        let mut cors_allowed_hosts = self.configuration.cors_allowed_hosts.clone();

        let agent_pool: Data<AgentControllerPool> = Data::from(self.agent_controller_pool.clone());

        #[cfg(feature = "web_admin_panel")]
        if let Some(web_admin_panel_config) = &self.web_admin_panel_service_configuration {
            cors_allowed_hosts.push(format!("http://{}", web_admin_panel_config.addr));
        }

        let buffered_request_manager: Data<BufferedRequestManager> =
            Data::from(self.buffered_request_manager.clone());
        let cors_allowed_hosts_arc = Arc::new(cors_allowed_hosts);
        let generate_tokens_sender_collection: Data<GenerateTokensSenderCollection> =
            Data::from(self.generate_tokens_sender_collection.clone());
        let model_metadata_sender_collection: Data<ModelMetadataSenderCollection> =
            Data::from(self.model_metadata_sender_collection.clone());
        let state_database: Data<dyn StateDatabase> = Data::from(self.state_database.clone());

        HttpServer::new(move || {
            App::new()
                .wrap(create_cors_middleware(cors_allowed_hosts_arc.clone()))
                .app_data(agent_pool.clone())
                .app_data(buffered_request_manager.clone())
                .app_data(generate_tokens_sender_collection.clone())
                .app_data(model_metadata_sender_collection.clone())
                .app_data(state_database.clone())
                .configure(common_http_route::get_health::register)
                .configure(http_route::api::get_agents::register)
                .configure(http_route::api::get_agents_stream::register)
                .configure(http_route::api::get_balancer_desired_state::register)
                .configure(http_route::api::get_buffered_requests::register)
                .configure(http_route::api::get_buffered_requests_stream::register)
                .configure(http_route::api::get_model_metadata::register)
                .configure(http_route::api::put_balancer_desired_state::register)
                .configure(http_route::api::ws_agent_socket::register)
                .configure(http_route::get_metrics::register)
        })
        .shutdown_signal(async move {
            if let Err(err) = shutdown.recv().await {
                error!("Failed to receive shutdown signal: {err}");
            }
        })
        .bind(self.configuration.addr)
        .expect("Unable to bind server to address")
        .run()
        .await?;

        Ok(())
    }
}
