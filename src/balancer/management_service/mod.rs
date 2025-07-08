pub mod configuration;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use async_trait::async_trait;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;

#[cfg(feature = "supervisor")]
use crate::balancer::fleet_management_database::FleetManagementDatabase;
use crate::balancer::http_route;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
#[cfg(feature = "supervisor")]
use crate::balancer::supervisor_controller_pool::SupervisorControllerPool;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
#[cfg(feature = "web_dashboard")]
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;

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
    #[cfg(feature = "supervisor")]
    fleet_management_database: Arc<dyn FleetManagementDatabase>,
    upstream_peers: Arc<UpstreamPeerPool>,
    #[cfg(feature = "web_dashboard")]
    web_dashboard_service_configuration: Option<WebDashboardServiceConfiguration>,
}

impl ManagementService {
    pub fn new(
        configuration: ManagementServiceConfiguration,
        #[cfg(feature = "supervisor")] fleet_management_database: Arc<dyn FleetManagementDatabase>,
        upstream_peers: Arc<UpstreamPeerPool>,
        #[cfg(feature = "web_dashboard")] web_dashboard_service_configuration: Option<
            WebDashboardServiceConfiguration,
        >,
    ) -> Self {
        ManagementService {
            configuration,
            fleet_management_database,
            upstream_peers,
            #[cfg(feature = "web_dashboard")]
            web_dashboard_service_configuration,
        }
    }
}

#[async_trait]
impl Service for ManagementService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut _shutdown: ShutdownWatch,
        _listeners_per_fd: usize,
    ) {
        #[allow(unused_mut)]
        let mut cors_allowed_hosts = self.configuration.cors_allowed_hosts.clone();

        #[cfg(feature = "supervisor")]
        let supervisor_pool: Data<SupervisorControllerPool> =
            Data::new(SupervisorControllerPool::new());

        #[cfg(feature = "web_dashboard")]
        if let Some(web_dashboard_config) = &self.web_dashboard_service_configuration {
            cors_allowed_hosts.push(format!("http://{}", web_dashboard_config.addr));
        }

        let cors_allowed_hosts_arc = Arc::new(cors_allowed_hosts);
        let fleet_management_database: Data<dyn FleetManagementDatabase> =
            Data::from(self.fleet_management_database.clone());
        let fleet_management_enable = self.configuration.fleet_management_enable;
        let metrics_endpoint_enable = self.configuration.metrics_endpoint_enable;
        let upstream_peers: Data<UpstreamPeerPool> = self.upstream_peers.clone().into();

        HttpServer::new(move || {
            #[allow(unused_mut)]
            let mut app = App::new()
                .wrap(create_cors_middleware(cors_allowed_hosts_arc.clone()))
                .app_data(upstream_peers.clone())
                .configure(http_route::api::get_agents::register)
                .configure(http_route::api::get_agents_stream::register)
                .configure(http_route::api::post_agent_status_update::register);

            #[cfg(feature = "supervisor")]
            if fleet_management_enable {
                app = app
                    .app_data(fleet_management_database.clone())
                    .app_data(supervisor_pool.clone())
                    .configure(http_route::api::get_supervisors::register)
                    .configure(http_route::api::ws_supervisor::register);
            }

            if metrics_endpoint_enable {
                app = app.configure(http_route::api::get_metrics::register)
            }

            app
        })
        .bind(self.configuration.addr)
        .expect("Unable to bind server to address")
        .run()
        .await
        .expect("Server unexpectedly stopped");
    }

    fn name(&self) -> &str {
        "balancer::management"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
