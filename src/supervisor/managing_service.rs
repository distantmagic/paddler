use actix_web::web::Bytes;
use async_trait::async_trait;
use log::{debug, error, info};
use pingora::{server::ShutdownWatch, services::Service};
use std::net::SocketAddr;
use tokio::{
    sync::broadcast::Sender,
    time::{interval, Duration, MissedTickBehavior},
};
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::errors::result::Result;

pub struct ManagingService {
    supervisor_management_addr: String,
}

impl ManagingService {
    pub fn new(supervisor_management_addr: SocketAddr) -> Result<Self> {
        let agent_id = Uuid::new_v4();

        Ok(ManagingService {
            supervisor_management_addr: format!(
                "http://{}/",
                supervisor_management_addr.to_string()
            ),
        })
    }
}

#[async_trait]
impl Service for ManagingService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        let mut ticker = interval(Duration::from_secs(1));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down reporting service");
                    return;
                },
            }
        }
    }

    fn name(&self) -> &str {
        "applying"
    }

    fn threads(&self) -> Option<usize> {
        None
    }
}
