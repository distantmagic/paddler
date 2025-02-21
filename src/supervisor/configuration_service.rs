use async_trait::async_trait;
use etcd_client::Client;
use log::{debug, error};
use pingora::{server::ShutdownWatch, services::Service};

#[cfg(unix)]
use pingora::server::ListenFds;
use tokio::sync::broadcast::Receiver;

use crate::errors::result::Result;

pub struct ConfigurationService {
    update_config: Receiver<Vec<String>>, // addr: SocketAddr,
}

impl ConfigurationService {
    pub fn new(update_config: Receiver<Vec<String>>) -> Result<Self> {
        Ok(ConfigurationService { update_config })
    }

    async fn persist_config(mut client: Client, args: Vec<String>) -> Result<()> {
        let json_vec = serde_json::to_string(&args)?;
        client.put("v1", json_vec, None).await?;

        Ok(())
    }
}

#[async_trait]
impl Service for ConfigurationService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        let client = Client::connect(["localhost:2379"], None).await;

        match client {
            Ok(mut client) => match client.put("foo", "bar", None).await {
                Ok(_) => loop {
                    tokio::select! {
                        _ = shutdown.changed() => {
                            debug!("Shutting down supervising service");
                            return;
                        },
                        args = self.update_config.recv() => {
                            match args {
                                Ok(args) => ConfigurationService::persist_config(client.clone(), args)
                                    .await
                                    .unwrap_or_else(|err| error!("Error while persisting llama.cpp configuration: {}", err)),
                                Err(err) => error!("Failed to receive llamacpp configuration: {}", err),
                            }
                        },
                    }
                },
                Err(err) => error!("Error while setting default arguments: {:#?}", err),
            },
            Err(err) => error!("Error while connecting with etcd server: {:#?}", err),
        }
    }

    fn name(&self) -> &str {
        "configuration"
    }

    fn threads(&self) -> Option<usize> {
        None
    }
}
