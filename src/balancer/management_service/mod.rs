pub mod configuration;

use std::sync::Arc;

use actix_web::middleware::from_fn;
use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use async_trait::async_trait;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;

use crate::balancer::http_route;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

pub struct ManagementService {
    configuration: ManagementServiceConfiguration,
    upstream_peers: Arc<UpstreamPeerPool>,
}

impl ManagementService {
    pub fn new(
        configuration: ManagementServiceConfiguration,
        upstream_peers: Arc<UpstreamPeerPool>,
    ) -> Self {
        ManagementService {
            configuration,
            upstream_peers,
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
        let upstream_peers: Data<UpstreamPeerPool> = self.upstream_peers.clone().into();

        HttpServer::new(move || {
            App::new()
                .app_data(upstream_peers.clone())
                .configure(http_route::api::get_agents::register)
                .configure(http_route::api::get_agents_stream::register)
                .configure(http_route::api::post_agent_status_update::register)
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
