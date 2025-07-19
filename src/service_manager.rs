use std::sync::Arc;

use anyhow::Result;
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

        for mut service in self.services {
            let service_name = service.name().to_string();
            let shutdown_broadcast_tx_arc_clone = shutdown_broadcast_tx_arc.clone();

            actix_rt::spawn(async move {
                loop {
                    info!("{service_name}: Starting");

                    if let Err(err) = service
                        .run(shutdown_broadcast_tx_arc_clone.subscribe())
                        .await
                    {
                        error!("{service_name}: {err}");
                    }

                    info!("{service_name}: Stopped");
                }
            });
        }

        let _ = shutdown_rx.await;
        let _ = shutdown_broadcast_tx.send(());

        Ok(())
    }
}
