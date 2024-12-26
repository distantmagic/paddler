use actix_web::{
    web::{Bytes, Data},
    App, HttpServer,
};
use async_trait::async_trait;
use pingora::{server::ShutdownWatch, services::Service};
use std::net::SocketAddr;
use tokio::sync::broadcast::Sender;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{errors::result::Result, supervisor::http_route};

pub struct ManagingService {
    supervisor_management_addr: String,
    status_update_tx: Sender<String>,
}

impl ManagingService {
    pub fn new(
        supervisor_management_addr: SocketAddr,
        status_update_tx: Sender<String>,
    ) -> Result<Self> {
        Ok(ManagingService {
            supervisor_management_addr: supervisor_management_addr.to_string(),
            status_update_tx,
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
        let status_update_tx = Data::new(self.status_update_tx.clone());

        HttpServer::new(move || {
            let app = App::new()
                .app_data(status_update_tx.clone())
                .configure(http_route::model_path::register);

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
