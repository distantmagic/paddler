use actix_web::{web::Data, App, HttpServer};
use async_trait::async_trait;
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use std::net::SocketAddr;
use std::sync::Arc;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::balancer::http_route;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;

pub struct ManagementService {
    addr: SocketAddr,
    upstream_peers: Arc<UpstreamPeerPool>,
}

impl ManagementService {
    pub fn new(addr: SocketAddr, upstream_peers: Arc<UpstreamPeerPool>) -> Self {
        ManagementService {
            addr,
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
    ) {
        let upstream_peers: Data<UpstreamPeerPool> = self.upstream_peers.clone().into();

        HttpServer::new(move || {
            App::new()
                .app_data(upstream_peers.clone())
                .configure(http_route::receive_status_update::register)
        })
        .bind(self.addr)
        .expect("Unable to bind server to address")
        .run()
        .await
        .expect("Server unexpectedly stopped");
    }

    fn name(&self) -> &str {
        "management"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
