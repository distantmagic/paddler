#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use std::{
    net::SocketAddr,
    path::Path,
    process::{Child, Command, Stdio},
    str,
};

use async_trait::async_trait;
use log::{debug, error, info, warn};
use pingora::{server::ShutdownWatch, services::Service};
use tokio::{
    sync::broadcast::Receiver,
    time::{interval, Duration, MissedTickBehavior},
};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::errors::{app_error::AppError, result::Result};

pub struct ApplyingService {
    addr: String,
    llama_path: String,
    model_path: String,
    monitoring_interval: Duration,
    llama_process: Option<Child>,
    update_model: Receiver<String>,
    update_binary: Receiver<String>,
    update_addr: Receiver<String>,
}

impl ApplyingService {
    pub fn new(
        addr: SocketAddr,
        llama_path: String,
        model_path: String,
        monitoring_interval: Duration,
        update_model: Receiver<String>,
        update_binary: Receiver<String>,
        update_addr: Receiver<String>,
    ) -> Result<Self> {
        Ok(ApplyingService {
            addr: addr.to_string(),
            llama_path,
            model_path,
            monitoring_interval,
            llama_process: None,
            update_model,
            update_binary,
            update_addr,
        })
    }

    async fn start_llamacpp_server(&mut self) -> Result<()> {
        let mut cmd = Command::new(self.llama_path.to_owned());

        if !is_a_gguf_file(self.model_path.to_string()) {
            return Err(AppError::InvalidFileError(
                "Insert a an existent gguf file for a model.".to_owned(),
            ));
        }

        let port = get_port(self.addr.clone());
        let host = get_host(self.addr.clone());

        cmd.args(&[
            "-m",
            self.model_path.as_str(),
            "--host",
            &host,
            "--port",
            &port,
            "--slots",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null());

        detach_process(&mut cmd);

        let child = cmd.spawn()?;
        self.llama_process = Some(child);

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

        // Kill old running llamacpp if there is one

        unsafe {
            if let Ok(listeners) = listeners::get_processes_by_port(
                str::parse(get_port(self.addr.clone()).as_str()).unwrap_unchecked(),
            ) {
                for process in listeners {
                    kill_llamacpp_server(process.pid);
                }
            }
        }

        // Start a new llamacpp so we can have control over this child
        match self.start_llamacpp_server().await {
            Err(e) => {
                warn!(
                    "Failed to get control over running llamacpp instance: {}",
                    e
                );
            }
            Ok(_) => (),
        }

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
                        info!("Llamacpp server fell off. Restarting server");
                    }
                },
                input_path = self.update_model.recv() => {
                    match input_path {
                        Ok(path) => {
                            self.model_path = path;
                            match self.start_llamacpp_server().await {
                                Ok(_) => {info!("Model Path was updated. Restarting server");},
                                Err(e) => {warn!("Failed to start llama server. Changes were not applied {}", e);}
                            }
                        },
                        Err(e) => {
                            error!("Failed to receive model path: {}", e);
                        }
                    }
                },
                input_path = self.update_binary.recv() => {
                    match input_path {
                        Ok(path) => {
                            self.llama_path = path;
                            match self.start_llamacpp_server().await {
                                Ok(_) => {info!("Binary path was updated. Restarting server");},
                                Err(e) => {warn!("Failed to start llama server. Changes were not applied {}", e);}
                            }
                        },
                        Err(e) => {
                            error!("Failed to receive binary path: {}", e);
                        }
                    }
                }
                input_addr = self.update_addr.recv() => {
                    match input_addr {
                        Ok(addr) => {
                            if let Some(llama_process) = &mut self.llama_process {
                                let pid = llama_process.id();

                                kill_llamacpp_server(pid);

                                self.addr = addr;
                            }
                            match self.start_llamacpp_server().await {
                                Ok(_) => {info!("Address was updated. Restarting server")},
                                Err(e) => {warn!("Failed to start llama server. Changes were not applied {}", e)}
                            }
                        },
                        Err(e) => {
                            error!("Failed to receive binary path: {}", e);
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

fn get_host(addr: String) -> String {
    unsafe {
        addr.split(':')
            .nth(0)
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

fn kill_llamacpp_server(pid: u32) {
    unsafe {
        if libc::kill(-(pid as i32), libc::SIGKILL) != 0 {
            warn!("Failed to kill process group. Changes were not applied");
        }
    }
}

fn detach_process(cmd: &mut Command) {
    unsafe {
        #[cfg(unix)]
        cmd.pre_exec(|| {
            libc::setsid();

            Ok(())
        });

        #[cfg(target_family = "windows")]
        {
            const DETACHED_PROCESS: u32 = 0x00000008;
            cmd.creation_flags(DETACHED_PROCESS);
        }
    }
}
