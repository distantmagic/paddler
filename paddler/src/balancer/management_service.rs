use actix_web::{web::Data, App, HttpServer};
use async_trait::async_trait;
use pingora::{server::ShutdownWatch, services::Service};
use std::{net::SocketAddr, sync::Arc};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::balancer::{http_route, upstream_peer_pool::UpstreamPeerPool};

pub struct ManagementService {
    addr: SocketAddr,
    #[cfg(feature = "web_dashboard")]
    management_dashboard_enable: bool,
    upstream_peers: Arc<UpstreamPeerPool>,
}

impl ManagementService {
    pub fn new(
        addr: SocketAddr,
        #[cfg(feature = "web_dashboard")] management_dashboard_enable: bool,
        upstream_peers: Arc<UpstreamPeerPool>,
    ) -> Self {
        ManagementService {
            addr,
            #[cfg(feature = "web_dashboard")]
            management_dashboard_enable,
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
        #[cfg(feature = "web_dashboard")]
        let management_dashboard_enable = self.management_dashboard_enable;

        let upstream_peers: Data<UpstreamPeerPool> = self.upstream_peers.clone().into();

        HttpServer::new(move || {
            #[allow(unused_mut)]
            let mut app = App::new()
                .app_data(upstream_peers.clone())
                .configure(http_route::registered_agents::register)
                .configure(http_route::receive_status_update::register);

            #[cfg(feature = "web_dashboard")]
            if management_dashboard_enable {
                app = app
                    .configure(http_route::dashboard::register)
                    .configure(http_route::static_files::register);
            }

            app
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
