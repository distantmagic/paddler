use std::sync::Arc;

use anyhow::Result;
use futures::future::join_all;
use log::error;
use log::info;
use tokio::sync::broadcast;
use tokio::sync::oneshot;

use crate::service::Service;

pub struct ServiceManager {
    services: Vec<Box<dyn Service>>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    pub fn add_service<TService: Service>(&mut self, service: TService) {
        self.services.push(Box::new(service));
    }

    pub async fn run_forever(self, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        let (shutdown_broadcast_tx, _) = broadcast::channel::<()>(1);
        let shutdown_broadcast_tx_arc = Arc::new(shutdown_broadcast_tx.clone());
        let mut service_handles = Vec::with_capacity(self.services.len());

        for mut service in self.services {
            let service_name = service.name().to_string();
            let shutdown_broadcast_tx_arc_clone = shutdown_broadcast_tx_arc.clone();

            service_handles.push(actix_rt::spawn(async move {
                loop {
                    info!("{service_name}: Starting");

                    let mut service_shutdown_rx = shutdown_broadcast_tx_arc_clone.subscribe();
                    let mut manager_shutdown_rx = shutdown_broadcast_tx_arc_clone.subscribe();

                    tokio::select! {
                        _ = manager_shutdown_rx.recv() => {
                            info!("{service_name}: Received shutdown signal");
                            break;
                        }
                        result = service.run(service_shutdown_rx) => {
                            match result {
                                Ok(()) => {
                                    info!("{service_name}: Stopped");
                                    break;
                                }
                                Err(err) => error!("{service_name}: {err}"),
                            }
                        }
                    }
                }
            }));
        }

        shutdown_rx.await?;
        shutdown_broadcast_tx.send(())?;
        join_all(service_handles).await;

        Ok(())
    }
}
