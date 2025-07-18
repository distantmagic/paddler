use anyhow::Result;
use log::debug;
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
        let (broadcast_tx, _) = broadcast::channel::<()>(1);

        for mut service in self.services {
            let service_name = service.name().to_string();
            let shutdown_subscriber = broadcast_tx.subscribe();

            actix_rt::spawn(async move {
                debug!("Starting service: {service_name}");

                if let Err(err) = service.run(shutdown_subscriber).await {
                    panic!("Service error: {err}");
                }

                debug!("Service stopped gracefully: {service_name}");
            });
        }

        let _ = shutdown_rx.await;
        let _ = broadcast_tx.send(());

        Ok(())
    }
}
