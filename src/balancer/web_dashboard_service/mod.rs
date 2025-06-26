pub mod configuration;
pub mod http_route;

use std::sync::Arc;

use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use async_trait::async_trait;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;

use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;

pub struct WebDashboardService {
    configuration: WebDashboardServiceConfiguration,
    upstream_peers: Arc<UpstreamPeerPool>,
}

impl WebDashboardService {
    pub fn new(
        configuration: WebDashboardServiceConfiguration,
        upstream_peers: Arc<UpstreamPeerPool>,
    ) -> Self {
        WebDashboardService {
            configuration,
            upstream_peers,
        }
    }
}

#[async_trait]
impl Service for WebDashboardService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut _shutdown: ShutdownWatch,
        _listeners_per_fd: usize,
    ) {
        let upstream_peers: Data<UpstreamPeerPool> = self.upstream_peers.clone().into();
        let configuration: Data<WebDashboardServiceConfiguration> =
            Data::new(self.configuration.clone());

        HttpServer::new(move || {
            App::new()
                .app_data(configuration.clone())
                .app_data(upstream_peers.clone())
                .configure(http_route::dashboard::register)
                .configure(http_route::static_files::register)
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
