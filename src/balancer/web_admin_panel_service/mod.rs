pub mod configuration;
pub mod http_route;
pub mod template_data;

use std::sync::Arc;

use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Result;
use async_trait::async_trait;
use log::error;
use tokio::sync::broadcast;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::web_admin_panel_service::template_data::TemplateData;
use crate::balancer::web_admin_panel_service::configuration::Configuration as WebAdminPanelServiceConfiguration;
use crate::service::Service;

pub struct WebAdminPanelService {
    agent_controller_pool: Arc<AgentControllerPool>,
    configuration: WebAdminPanelServiceConfiguration,
}

impl WebAdminPanelService {
    pub fn new(
        agent_controller_pool: Arc<AgentControllerPool>,
        configuration: WebAdminPanelServiceConfiguration,
    ) -> Self {
        WebAdminPanelService {
            agent_controller_pool,
            configuration,
        }
    }
}

#[async_trait]
impl Service for WebAdminPanelService {
    fn name(&self) -> &'static str {
        "balancer::web_admin_panel_service"
    }

    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let agent_controller_pool: Data<AgentControllerPool> = Data::from(self.agent_controller_pool.clone());
        let template_data: Data<TemplateData> = Data::new(self.configuration.template_data.clone());

        HttpServer::new(move || {
            App::new()
                .app_data(agent_controller_pool.clone())
                .app_data(template_data.clone())
                .configure(http_route::favicon::register)
                .configure(http_route::static_files::register)
                .configure(http_route::home::register)
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
