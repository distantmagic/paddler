use async_trait::async_trait;
use log::{debug, error, info, warn};
use pingora::{server::ShutdownWatch, services::Service};
use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    process::{Child, Command, Stdio},
};
use toml_edit::DocumentMut;

#[cfg(feature = "etcd")]
use etcd_client::Client;

#[cfg(feature = "etcd")]
use std::net::SocketAddr;
use tokio::sync::broadcast::{Receiver, Sender};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{errors::result::Result, ConfigDriver};

#[cfg(feature = "etcd")]
use crate::errors::app_error::AppError;

pub struct ApplicationService {
    working_args: (Option<Vec<String>>, Option<Vec<String>>),
    llamacpp_process: Option<Child>,
    update_llamacpp: Receiver<Vec<String>>,
    update_config: Sender<Vec<String>>,
}

impl ApplicationService {
    pub fn new(
        binary: String,
        model: String,
        port: u16,
        config_driver: ConfigDriver,
        update_llamacpp: Receiver<Vec<String>>,
        update_config: Sender<Vec<String>>,
    ) -> Result<Self> {
        let working_args = Self::get_default_config(config_driver, binary, model, port)?;

        Ok(ApplicationService {
            working_args,
            llamacpp_process: None,
            update_llamacpp,
            update_config,
        })
    }

    async fn start_llamacpp_server(&mut self) -> Result<()> {
        if let Some(args) = self.working_args.0.clone() {
            self.spawn_llama_process(&args).await?
        } else if let Some(old_args) = self.working_args.1.clone() {
            self.spawn_llama_process(&old_args).await?;
        }

        Ok(())
    }

    async fn spawn_llama_process(&mut self, args: &Vec<String>) -> Result<()> {
        let mut cmd = Command::new(&args[1]);
        cmd.args(&args[2..])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        match cmd.spawn() {
            Ok(child) => {
                if let Some(process) = &mut self.llamacpp_process {
                    process.kill()?;
                    process.wait()?;
                }
                self.llamacpp_process = Some(child);
                self.update_config.send(args.to_vec())?;
                Ok(())
            }
            Err(e) => {
                error!("Failed to start process: {}", e);
                warn!("Changes were not applied: {}", e);
                Err(e.into())
            }
        }
    }

    async fn handle_new_arguments(&mut self, args: Vec<String>) {
        let primary = self.working_args.0.take();
        self.working_args.0 = Some(args);

        match self.start_llamacpp_server().await {
            Ok(_) => {
                self.working_args.1 = primary;
                info!("Configuration updated and server restarted.");
            }
            Err(err) => {
                warn!("Failed to start server with new configuration: {}", err);
                self.working_args.0 = primary;
            }
        }
    }

    async fn server_is_running(&mut self) -> bool {
        match &mut self.llamacpp_process {
            Some(child) => match child.try_wait() {
                Ok(Some(_)) => return false,
                Ok(None) => return true,
                Err(e) => {
                    error!("Error checking process status: {}", e);
                    return false;
                }
            },
            None => return false,
        }
    }

    fn get_default_config(
        config_driver: ConfigDriver,
        binary: String,
        model: String,
        port: u16,
    ) -> Result<(Option<Vec<String>>, Option<Vec<String>>)> {
        let config = match config_driver {
            ConfigDriver::File { path, name } => load_file_config(path, name),
            #[cfg(feature = "etcd")]
            ConfigDriver::Etcd { addr, name: _ } => load_etcd_config(addr),
        };

        match config? {
            Some(config) => Ok((Some(config), None)),
            None => {
                let v1 = vec![
                    "--args".to_string(),
                    binary,
                    "-m".to_string(),
                    model,
                    "-np".to_string(),
                    "4".to_string(),
                    "--port".to_string(),
                    port.to_string(),
                    "--slots".to_string(),
                ];
                return Ok((Some(v1), None));
            }
        }
    }
}

fn load_file_config(file_path: PathBuf, name: String) -> Result<Option<Vec<String>>> {
    if let Ok(mut file) = File::open(file_path) {
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;

        let config = file_content.parse::<DocumentMut>()?;

        let config = config
            .get("config")
            .and_then(|config| config.get(name))
            .and_then(|config| config.as_array());

        if let Some(config) = config {
            let config: Vec<String> = config
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            return Ok(Some(config));
        }
    }

    Ok(None)
}

#[cfg(feature = "etcd")]
fn load_etcd_config(etcd_address: SocketAddr) -> Result<Option<Vec<String>>> {
    let runtime = tokio::runtime::Runtime::new()?;

    runtime.block_on(async {
        let _ = match Client::connect([etcd_address.to_string()], None).await {
            Ok(mut client) => match client.get("v1", None).await {
                Ok(response) => match response.kvs().first() {
                    Some(v1) => {
                        let v1 = serde_json::from_str::<Vec<String>>(v1.value_str()?)?;

                        Ok::<std::option::Option<Vec<std::string::String>>, AppError>(Some(v1))
                    }
                    None => {
                        error!("Failed while parsing configuration file");
                        return Ok(None);
                    }
                },
                Err(_) => {
                    error!("Failed while connecting to etcd server. Is it running?");
                    return Ok(None);
                }
            },
            Err(_) => {
                error!("Failed while connecting to etcd server. Is it running?");
                return Ok(None);
            }
        };

        Ok(None)
    })
}

#[async_trait]
impl Service for ApplicationService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        let mut receiver = self.update_llamacpp.resubscribe();

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down supervising service");
                    return;
                },
                running = self.server_is_running() => {
                    if !running {
                        match self.start_llamacpp_server().await {
                            Ok(()) => info!("Llamacpp server restarted."),
                            Err(err) => error!("Failed to start llama server: {}", err)
                        }
                    }
                },
                args = receiver.recv() => {
                    match args {
                        Ok(new_args) => {
                            self.handle_new_arguments(new_args).await;
                        },
                        Err(e) => {
                            error!("Failed to receive llamacpp configuration: {}", e);
                        }
                    }
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
