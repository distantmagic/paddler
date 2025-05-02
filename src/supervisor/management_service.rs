use actix_web::{web::Data, App, HttpServer};
use async_trait::async_trait;
use log::{debug, info};
use pingora::{server::ShutdownWatch, services::Service};
use serde_json::{Map, Value};
use tokio::signal::unix::{signal, SignalKind};

use std::{
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc, Mutex},
    time::{Duration, Instant},
};
use throttle::Throttle;
use tokio::sync::broadcast::Sender;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{errors::result::Result, supervisor::http_route};

pub struct ManagementService {
    supervisor_addr: String,
    state: Data<State>,
}

pub struct State {
    pub update_llamacpp: Sender<Vec<String>>,
    pub throttle: Mutex<Throttle>,
    pub args: Mutex<Vec<Map<String, Value>>>,
    pub last_request: Arc<Mutex<Option<Instant>>>,
    pub is_throttle_running: AtomicBool,
}

impl ManagementService {
    pub fn new(supervisor_addr: SocketAddr, update_llamacpp: Sender<Vec<String>>) -> Result<Self> {
        Ok(ManagementService {
            supervisor_addr: supervisor_addr.to_string(),
            state: Data::new(State {
                update_llamacpp,
                throttle: Mutex::new(Throttle::new(Duration::from_millis(600), 20)),
                args: Mutex::new(Vec::new()),
                last_request: Arc::new(Mutex::new(None)),
                is_throttle_running: AtomicBool::new(false),
            }),
        })
    }
}

#[async_trait]
impl Service for ManagementService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        let supervisor_addr = self.supervisor_addr.clone();
        let state = self.state.clone();

        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();

        HttpServer::new(move || {
            App::new()
                .app_data(state.clone())
                .configure(http_route::receive_update::register)
        })
        .bind(supervisor_addr)
        .expect("Unable to bind server to address")
        .run()
        .await
        .expect("Server unexpectedly stopped");

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down configuration service");
                    return;
                },
                _ = sigint.recv() => {
                    info!("Received SIGINT, shutting down next release observer service");
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, shutting down next release observer service");
                }
            }
        }
    }

    fn name(&self) -> &str {
        "management"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
