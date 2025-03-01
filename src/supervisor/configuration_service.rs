#[cfg(feature = "etcd")]
use etcd_client::Client;
#[cfg(feature = "etcd")]
use std::net::SocketAddr;

use async_trait::async_trait;
use log::{debug, error};
use pingora::{server::ShutdownWatch, services::Service};
use std::{fs, path::PathBuf};
use toml::Value;

#[cfg(unix)]
use pingora::server::ListenFds;
use tokio::sync::broadcast::Receiver;

use crate::errors::result::Result;

pub struct ConfigurationService {
    update_config: Receiver<Vec<String>>,
    #[cfg(feature = "etcd")]
    etcd_address: Option<SocketAddr>,
    file_path: Option<PathBuf>,
}

impl ConfigurationService {
    pub fn new(
        update_config: Receiver<Vec<String>>,
        #[cfg(feature = "etcd")] etcd_address: Option<SocketAddr>,
        file_path: Option<PathBuf>,
    ) -> Result<Self> {
        Ok(ConfigurationService {
            update_config,
            #[cfg(feature = "etcd")]
            etcd_address,
            file_path,
        })
    }

    async fn persist_config(&self, args: Vec<String>) -> Result<()> {
        if let Some(file_path) = &self.file_path {
            let config_value = toml::Value::Table({
                let mut table = toml::value::Table::new();
                let vec_toml_value: Vec<Value> =
                    args.clone().into_iter().map(Value::String).collect();
                table.insert("v1".to_string(), toml::Value::Array(vec_toml_value));
                table
            });

            let mut toml_table = toml::value::Table::new();
            toml_table.insert("config".to_string(), config_value);

            let toml_string = toml::to_string(&toml_table)?;
            fs::write(file_path, toml_string)?;
        }

        #[cfg(feature = "etcd")]
        if let Some(etcd_address) = &self.etcd_address {
            let mut client = Client::connect([etcd_address.to_string()], None).await?;
            let json_vec = serde_json::to_string(&args)?;
            client.put("v1", json_vec, None).await?;
        }

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
        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down configuration service");
                    return;
                },
                args = self.update_config.recv() => {
                    match args {
                        Ok(args) => {
                            if let Err(err) = self.persist_config(args).await {
                                error!("Error while persisting configuration: {}", err);
                            }
                        },
                        Err(err) => error!("Failed to receive configuration: {}", err),
                    }
                },
            }
        }
    }

    fn name(&self) -> &str {
        "configuration"
    }

    fn threads(&self) -> Option<usize> {
        None
    }
}
