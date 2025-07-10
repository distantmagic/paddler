pub mod configuration;
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
use crate::balancer::fleet_management_database::FleetManagementDatabase;
use crate::balancer::http_route;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;
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

pub struct ManagementService {
    configuration: ManagementServiceConfiguration,
    fleet_management_database: Arc<dyn FleetManagementDatabase>,
    // upstream_peers: Arc<UpstreamPeerPool>,
    #[cfg(feature = "web_dashboard")]
    web_dashboard_service_configuration: Option<WebDashboardServiceConfiguration>,
}

impl ManagementService {
    pub fn new(
        configuration: ManagementServiceConfiguration,
        fleet_management_database: Arc<dyn FleetManagementDatabase>,
        #[cfg(feature = "web_dashboard")] web_dashboard_service_configuration: Option<
            WebDashboardServiceConfiguration,
        >,
    ) -> Self {
        ManagementService {
            configuration,
            fleet_management_database,
            // upstream_peers,
            #[cfg(feature = "web_dashboard")]
            web_dashboard_service_configuration,
        }
    }
}

#[async_trait]
impl Service for ManagementService {
    async fn run(&mut self, mut _shutdown: broadcast::Receiver<()>) -> Result<()> {
        #[allow(unused_mut)]
        let mut cors_allowed_hosts = self.configuration.cors_allowed_hosts.clone();

        let agent_pool: Data<AgentControllerPool> = Data::new(AgentControllerPool::new());

        #[cfg(feature = "web_dashboard")]
        if let Some(web_dashboard_config) = &self.web_dashboard_service_configuration {
            cors_allowed_hosts.push(format!("http://{}", web_dashboard_config.addr));
        }

        let cors_allowed_hosts_arc = Arc::new(cors_allowed_hosts);
        let fleet_management_database: Data<dyn FleetManagementDatabase> =
            Data::from(self.fleet_management_database.clone());
        let fleet_management_enable = self.configuration.fleet_management_enable;
        let metrics_endpoint_enable = self.configuration.metrics_endpoint_enable;

        Ok(HttpServer::new(move || {
            #[allow(unused_mut)]
            let mut app = App::new().wrap(create_cors_middleware(cors_allowed_hosts_arc.clone()));

            if fleet_management_enable {
                app = app
                    .app_data(fleet_management_database.clone())
                    .app_data(agent_pool.clone())
                    .configure(http_route::api::ws_agent::register);
            }

            // if metrics_endpoint_enable {
            //     app = app.configure(http_route::api::get_metrics::register)
            // }

            app
        })
        .bind(self.configuration.addr)
        .expect("Unable to bind server to address")
        .run()
        .await?)
    }
}
