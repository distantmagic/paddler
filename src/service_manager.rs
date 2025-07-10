use std::thread::spawn;

use actix_rt::System;
use anyhow::Result;
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
        let mut thread_handles = Vec::new();

        for mut service in self.services {
            let shutdown_subscriber = broadcast_tx.subscribe();

            let handle = spawn(move || {
                System::new().block_on(async move {
                    if let Err(err) = service.run(shutdown_subscriber).await {
                        panic!("Service error: {err}");
                    }
                });
            });

            thread_handles.push(handle);
        }

        let _ = shutdown_rx.await;
        let _ = broadcast_tx.send(());

        for handle in thread_handles {
            let _ = handle.join();
        }

        Ok(())
    }
}
