use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::SinkExt;
use futures_util::StreamExt;
use log::debug;
use log::error;
use log::info;
#[cfg(unix)]
use pingora::server::ListenFds;
use pingora::server::ShutdownWatch;
use pingora::services::Service;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

use crate::supervisor::reconciliation_queue::ReconciliationQueue;

pub struct ManagementSocketClientService {
    management_addr: SocketAddr,
    name: Option<String>,
    reconciliation_queue: Arc<ReconciliationQueue>,
    status_endpoint_url: String,
}

impl ManagementSocketClientService {
    pub fn new(
        management_addr: SocketAddr,
        name: Option<String>,
        reconciliation_queue: Arc<ReconciliationQueue>,
    ) -> Result<Self> {
        let supervisor_id = Uuid::new_v4();

        Ok(ManagementSocketClientService {
            management_addr,
            name,
            reconciliation_queue,
            status_endpoint_url: format!(
                "ws://{management_addr}/api/v1/supervisor_socket/{supervisor_id}"
            ),
        })
    }

    async fn keep_connection_alive(&self) -> Result<()> {
        let (ws_stream, _response) = connect_async(self.status_endpoint_url.clone()).await?;

        info!(
            "Connected to management server at {}",
            self.status_endpoint_url
        );

        let (mut write, mut read) = ws_stream.split();

        // Send a message
        write
            .send(Message::Text("Hello from client".into()))
            .await?;

        while let Some(msg) = read.next().await {
            match msg? {
                Message::Text(text) => {
                    println!("Received: {text}");
                }
                Message::Binary(_) => {
                    error!("Received binary message, which is not expected");
                }
                Message::Close(_) => {
                    println!("Connection closed");
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Service for ManagementSocketClientService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
        _listeners_per_fd: usize,
    ) {
        let mut ticker = interval(Duration::from_secs(1));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down monitoring service");
                    return;
                },
                _ = ticker.tick() => {
                    if let Err(err) = self.keep_connection_alive().await {
                        error!("Failed to keep the connection alive: {err}");
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "supervisor::management_socket_client"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
