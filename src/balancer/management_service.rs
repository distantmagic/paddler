use actix_web::{App, HttpServer};
use async_trait::async_trait;
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use std::net::SocketAddr;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::balancer::http_route;

pub struct ManagementService {
    addr: SocketAddr,
}

impl ManagementService {
    pub fn new(addr: SocketAddr) -> Self {
        ManagementService { addr }
    }
}

#[async_trait]
impl Service for ManagementService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut _shutdown: ShutdownWatch,
    ) {
        println!("Starting HTTP service");

        HttpServer::new(|| App::new().configure(http_route::receive_status_update::register))
            .bind(self.addr)
            .expect("Unable to bind server to address")
            .run()
            .await
            .expect("Server unexpectedly stopped");
    }

    fn name(&self) -> &str {
        "balancer"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
