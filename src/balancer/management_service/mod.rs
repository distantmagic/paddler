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

use crate::balancer::http_route;
use crate::balancer::management_service::configuration::Configuration as ManagementServiceConfiguration;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

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
        let cors_allowed_hosts = Arc::new(self.configuration.cors_allowed_hosts.clone());
        let upstream_peers: Data<UpstreamPeerPool> = self.upstream_peers.clone().into();

        HttpServer::new(move || {
            App::new()
                .wrap(create_cors_middleware(cors_allowed_hosts.clone()))
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
