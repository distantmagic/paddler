use actix_web::{web::Data, App, HttpServer};
use async_trait::async_trait;
use pingora::{server::ShutdownWatch, services::Service};
use tokio::sync::broadcast::Sender;
use std::net::SocketAddr;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{errors::result::Result, supervisor::http_route};

pub struct ManagingService {
    supervisor_management_addr: String,
    update_llamacpp_tx: Sender<String>
}

impl ManagingService {
    pub fn new(
        supervisor_management_addr: SocketAddr,
        update_llamacpp_tx: Sender<String>,
    ) -> Result<Self> {
        Ok(ManagingService {
            supervisor_management_addr: supervisor_management_addr.to_string(),
            update_llamacpp_tx,
        })
    }
}

#[async_trait]
impl Service for ManagingService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut _shutdown: ShutdownWatch,
    ) {
        let update_llamacpp_tx = Data::new(self.update_llamacpp_tx.clone());

        HttpServer::new(move || {
            let app = App::new()
                .app_data(update_llamacpp_tx.clone())
                .configure(http_route::update::register);

            app
        })
        .bind(self.supervisor_management_addr.to_owned())
        .expect("Unable to bind server to address")
        .run()
        .await
        .expect("Server unexpectedly stopped");
    }

    fn name(&self) -> &str {
        "applying"
    }

    fn threads(&self) -> Option<usize> {
        None
    }
}
