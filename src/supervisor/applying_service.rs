use std::{
    net::SocketAddr,
    os::unix::process::CommandExt,
    path::Path,
    process::{Child, Command, Stdio},
    str,
};

use actix_web::web::Bytes;
use async_trait::async_trait;
use log::warn;
use log::{debug, error};
use pingora::{server::ShutdownWatch, services::Service};
use tokio::{
    sync::broadcast::Receiver,
    time::{interval, Duration, MissedTickBehavior},
};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::errors::{app_error::AppError, result::Result};

pub struct ApplyingService {
    port: String,
    llama_server_path: String,
    model_path: String,
    monitoring_interval: Duration,
    llama_process: Option<Child>,
    status_update_rx: Receiver<Bytes>,
}

impl ApplyingService {
    pub fn new(
        addr: SocketAddr,
        llama_server_path: String,
        model_path: String,
        monitoring_interval: Duration,
        status_update_rx: Receiver<Bytes>,
    ) -> Result<Self> {
        let port = get_port(addr.to_string());
        Ok(ApplyingService {
            port,
            llama_server_path,
            model_path,
            monitoring_interval,
            llama_process: None,
            status_update_rx,
        })
    }

    async fn start_llamacpp_server(&mut self) -> Result<()> {
        unsafe {
            let mut cmd = Command::new(self.llama_server_path.to_owned());

            if !is_a_gguf_file(self.model_path.to_string()) {
                return Err(AppError::InvalidFileError(
                    "Insert a Valid gguf file for a model.".to_owned(),
                ));
            }

            cmd.args(&[
                "-m",
                self.model_path.as_str(),
                "--port",
                &self.port,
                "--slots",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null());

            #[cfg(unix)]
            cmd.pre_exec(|| {
                libc::setsid();

                Ok(())
            });

            let child = cmd.spawn()?;
            self.llama_process = Some(child);
        }

        Ok(())
    }

    fn server_is_running(&mut self) -> bool {
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
}

#[async_trait]
impl Service for ApplyingService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        let mut ticker = interval(self.monitoring_interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down supervising service");
                    return;
                },
                _ = ticker.tick() => {
                    if !self.server_is_running() {
                        if let Err(e) = self.start_llamacpp_server().await {
                            error!("Failed to start llama server: {}", e);
                        }
                        warn!("Llamacpp server fell off. Restarting server");
                    }
                },
                input_path = self.status_update_rx.recv() => {
                    match input_path {
                        Ok(bytes_path) => {
                            match str::from_utf8(&bytes_path) {
                                Ok(string_path) => {
                                    self.model_path = string_path.to_string();

                                    match self.start_llamacpp_server().await {
                                        Ok(_) => {warn!("Model Path was updated. Restarting server");},
                                        Err(e) => {error!("Failed to start llama server: {}", e);}
                                    }
                                },
                                Err(e) => {
                                    error!("Failed to receive parse path into a valid: {}", e);
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to receive model path: {}", e);
                        }
                    }
                }
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

fn get_port(addr: String) -> String {
    unsafe {
        addr.split(':')
            .nth(1)
            .unwrap_unchecked()
            .parse::<String>()
            .unwrap_unchecked()
    }
}

fn is_a_gguf_file(path: String) -> bool {
    let file = Path::new(&path);

    if file.exists() {
        if let Some(ext) = file.extension() {
            if ext.to_str() == Some("gguf") {
                return true;
            }

            return false;
        }
        return false;
    }

    false
}
