pub mod configuration;
pub mod http_route;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::generate_tokens_sender_collection::GenerateTokensSenderCollection;
use crate::balancer::inference_service::configuration::Configuration as InferenceServiceConfiguration;
use crate::balancer::state_database::StateDatabase;
#[cfg(feature = "web_admin_panel")]
use crate::balancer::web_admin_panel_service::configuration::Configuration as WebAdminPanelServiceConfiguration;
use crate::service::Service;

fn create_cors_middleware(allowed_hosts: Arc<Vec<String>>) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "OPTIONS"])
        .allowed_headers(vec![
            header::ACCEPT,
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
        ])
        .max_age(3600);

    for host in allowed_hosts.iter() {
        cors = cors.allowed_origin(host);
    }

    cors
}

pub struct InferenceService {
    agent_controller_pool: Arc<AgentControllerPool>,
    configuration: InferenceServiceConfiguration,
    generate_tokens_sender_collection: Arc<GenerateTokensSenderCollection>,
    state_database: Arc<dyn StateDatabase>,
    #[cfg(feature = "web_admin_panel")]
    web_admin_panel_service_configuration: Option<WebAdminPanelServiceConfiguration>,
}

impl InferenceService {
    pub fn new(
        agent_controller_pool: Arc<AgentControllerPool>,
        configuration: InferenceServiceConfiguration,
        generate_tokens_sender_collection: Arc<GenerateTokensSenderCollection>,
        state_database: Arc<dyn StateDatabase>,
        #[cfg(feature = "web_admin_panel")] web_admin_panel_service_configuration: Option<
            WebAdminPanelServiceConfiguration,
        >,
    ) -> Self {
        InferenceService {
            agent_controller_pool,
            configuration,
            generate_tokens_sender_collection,
            state_database,
            #[cfg(feature = "web_admin_panel")]
            web_admin_panel_service_configuration,
        }
    }
}

#[async_trait]
impl Service for InferenceService {
    fn name(&self) -> &'static str {
        "balancer::inference_service"
    }

    async fn run(&mut self, mut _shutdown: broadcast::Receiver<()>) -> Result<()> {
        #[allow(unused_mut)]
        let mut cors_allowed_hosts = self.configuration.cors_allowed_hosts.clone();

        let agent_pool: Data<AgentControllerPool> = Data::from(self.agent_controller_pool.clone());

        #[cfg(feature = "web_admin_panel")]
        if let Some(web_admin_panel_config) = &self.web_admin_panel_service_configuration {
            cors_allowed_hosts.push(format!("http://{}", web_admin_panel_config.addr));
        }

        let cors_allowed_hosts_arc = Arc::new(cors_allowed_hosts);
        let generate_tokens_sender_collection: Data<GenerateTokensSenderCollection> =
            Data::from(self.generate_tokens_sender_collection.clone());
        let state_database: Data<dyn StateDatabase> = Data::from(self.state_database.clone());

        Ok(HttpServer::new(move || {
            App::new()
                .wrap(create_cors_middleware(cors_allowed_hosts_arc.clone()))
                .app_data(agent_pool.clone())
                .app_data(generate_tokens_sender_collection.clone())
                .app_data(state_database.clone())
                .configure(http_route::api::ws_inference_socket::register)
        })
        .bind(self.configuration.addr)
        .expect("Unable to bind server to address")
        .run()
        .await?)
    }
}
