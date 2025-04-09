use async_trait::async_trait;
#[cfg(feature = "etcd")]
use etcd_client::Client;
use log::{debug, error};
use pingora::{server::ShutdownWatch, services::Service};
use std::{
    fs::{self, File},
    io::Read, path::PathBuf,
};
use toml_edit::{value, DocumentMut};

#[cfg(unix)]
use pingora::server::ListenFds;
use tokio::sync::broadcast::Receiver;

use crate::{errors::result::Result, ConfigDriver};

pub struct ConfigurationService {
    update_config: Receiver<Vec<String>>,
    config_driver: ConfigDriver,
}

impl ConfigurationService {
    pub fn new(update_config: Receiver<Vec<String>>, config_driver: ConfigDriver) -> Result<Self> {
        Ok(ConfigurationService {
            update_config,
            config_driver,
        })
    }

    async fn persist_config(&self, args: Vec<String>) -> Result<()> {
        match &self.config_driver {
            ConfigDriver::File { path, name } => {
                if !Self::is_a_toml_file(path) {
                    error!("File is not `.toml`. Configuration will not be persisted.")
                }

                let mut config = if let Ok(mut file) = File::open(path) {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    contents.parse::<DocumentMut>()?
                } else {
                    DocumentMut::new()
                };
                if !config.contains_table("config") {
                    config["config"] = toml_edit::table();
                }

                config["config"][name] = value(serde_json::to_string(&args)?);

                fs::write(path, config.to_string())?;
            }
            #[cfg(feature = "etcd")]
            ConfigDriver::Etcd { addr, name } => {
                let mut client = Client::connect([addr.to_string()], None).await?;
                let json_vec = serde_json::to_string(&args)?;
                client.put(name.as_str(), json_vec, None).await?;
            }
        }

        Ok(())
    }

    fn is_a_toml_file(path: &PathBuf) -> bool {
        path.extension().is_some_and(|x| x == "toml")
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
