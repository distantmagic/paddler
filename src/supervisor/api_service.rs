use std::net::SocketAddr;

use actix_web::App;
use actix_web::HttpServer;
use async_trait::async_trait;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;

use crate::supervisor::http_route;

pub struct ApiService {
    addr: SocketAddr,
}

impl ApiService {
    pub fn new(addr: SocketAddr) -> Self {
        ApiService {
            addr,
        }
    }
}

#[async_trait]
impl Service for ApiService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut _shutdown: ShutdownWatch,
        _listeners_per_fd: usize,
    ) {
        HttpServer::new(move || {
            App::new().configure(http_route::api::post_change_request::register)
        })
        .bind(self.addr)
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
