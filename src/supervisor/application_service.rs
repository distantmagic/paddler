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
    llama_process: Option<Child>,
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
            llama_process: None,
            update_llamacpp,
            update_config,
        })
    }

    async fn start_llamacpp_server(&mut self) -> Result<()> {
        if let Some(args) = self.working_args.0.clone() {
            if self.spawn_llama_process(&args).await.is_ok() {
                return Ok(());
            }
        }

        if let Some(old_args) = self.working_args.1.clone() {
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
                if let Some(process) = &mut self.llama_process {
                    let _ = process.kill();
                    let _ = process.wait();
                }
                self.llama_process = Some(child);
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

        if let Err(e) = self.start_llamacpp_server().await {
            warn!("Failed to start server with new configuration: {}", e);
            self.working_args.0 = primary;
        } else {
            self.working_args.1 = primary;
            info!("Configuration updated and server restarted.");
        }
    }

    async fn server_is_running(&mut self) -> bool {
        if let Some(child) = &mut self.llama_process {
            match child.try_wait() {
                Ok(Some(_)) => false,
                Ok(None) => true,
                Err(e) => {
                    error!("Error checking process status: {}", e);
                    false
                }
            }
        } else {
            false
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

        if let Some(config) = config? {
            Ok((Some(config), None))
        } else {
            let v1 = vec![
                "--args".to_string(),
                binary,
                "-m".to_string(),
                model,
                "--port".to_string(),
                port.to_string(),
                "--slots".to_string(),
            ];
            Ok((Some(v1), None))
        }
    }
}

fn load_file_config(file_path: PathBuf, name: String) -> Result<Option<Vec<String>>> {
    if let Ok(mut file) = File::open(file_path) {
        let mut file_content = String::new();
        file.read_to_string(&mut file_content)?;

        let config = file_content.parse::<DocumentMut>()?;

        if let Some(config) = config["config"][name].as_array() {
            eprintln!("{:#?}", config);
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
        #[cfg(feature = "etcd")]
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
                        if let Err(e) = self.start_llamacpp_server().await {
                            error!("Failed to start llama server: {}", e);
                        } else {
                            info!("Llamacpp server restarted.");
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
