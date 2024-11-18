use actix_web::{App, HttpServer};
use async_trait::async_trait;
use log::debug;
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use std::net::SocketAddr;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::balancer::http_route;

pub struct BalancingService {
}

impl BalancingService {
    pub fn new() -> Self {
        BalancingService {}
    }
}

#[async_trait]
impl Service for BalancingService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut _shutdown: ShutdownWatch,
    ) {
    }

    fn name(&self) -> &str {
        "balancing"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
