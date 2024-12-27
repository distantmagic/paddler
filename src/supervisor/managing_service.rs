use actix_web::{web::Data, App, HttpServer};
use async_trait::async_trait;
use pingora::{server::ShutdownWatch, services::Service};
use std::net::SocketAddr;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{cmd::supervisor::UpdateLlamacpp, errors::result::Result, supervisor::http_route};

pub struct ManagingService {
    supervisor_management_addr: String,
    update_channels: UpdateLlamacpp,
}

impl ManagingService {
    pub fn new(
        supervisor_management_addr: SocketAddr,
        update_channels: UpdateLlamacpp,
    ) -> Result<Self> {
        Ok(ManagingService {
            supervisor_management_addr: supervisor_management_addr.to_string(),
            update_channels,
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
        let update_channels = Data::new(self.update_channels.clone());

        HttpServer::new(move || {
            let app = App::new()
                .app_data(update_channels.clone())
                .configure(http_route::model_path::register)
                .configure(http_route::binary_path::register);

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
