pub mod configuration;
pub mod http_route;

use std::sync::Arc;

use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;
use crate::service::Service;

pub struct WebDashboardService {
    agent_controller_pool: Arc<AgentControllerPool>,
    configuration: WebDashboardServiceConfiguration,
}

impl WebDashboardService {
    pub fn new(
        agent_controller_pool: Arc<AgentControllerPool>,
        configuration: WebDashboardServiceConfiguration,
    ) -> Self {
        WebDashboardService {
            agent_controller_pool,
            configuration,
        }
    }
}

#[async_trait]
impl Service for WebDashboardService {
    async fn run(&mut self, mut _shutdown: broadcast::Receiver<()>) -> Result<()> {
        let agent_controller_pool: Data<AgentControllerPool> =
            Data::from(self.agent_controller_pool.clone());
        let configuration: Data<WebDashboardServiceConfiguration> =
            Data::new(self.configuration.clone());

        Ok(HttpServer::new(move || {
            App::new()
                .app_data(agent_controller_pool.clone())
                .app_data(configuration.clone())
                .configure(http_route::dashboard::register)
                .configure(http_route::static_files::register)
        })
        .bind(self.configuration.addr)
        .expect("Unable to bind server to address")
        .run()
        .await?)
    }
}
