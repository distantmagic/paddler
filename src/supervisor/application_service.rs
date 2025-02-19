use async_trait::async_trait;
use log::{debug, error, info, warn};
use pingora::{server::ShutdownWatch, services::Service};
use reqwest::Client;
use std::{
    net::SocketAddr,
    process::{Child, Command, Stdio},
};
use tokio::sync::broadcast::Receiver;

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::errors::result::Result;

pub struct ApplyingService {
    args: (Option<Vec<String>>, Option<Vec<String>>),
    llama_process: Option<Child>,
    update_llamacpp: Receiver<Vec<String>>,
    supervisor_addr: SocketAddr,
}

impl ApplyingService {
    pub fn new(
        args: Vec<String>,
        update_llamacpp: Receiver<Vec<String>>,
        supervisor_addr: SocketAddr,
    ) -> Result<Self> {
        Ok(ApplyingService {
            args: (Some(args), None),
            llama_process: None,
            update_llamacpp,
            supervisor_addr,
        })
    }

    async fn start_llamacpp_server(&mut self) -> Result<()> {
        if let Some(args) = self.args.0.clone() {
            if self.spawn_llama_process(&args).await.is_ok() {
                return Ok(());
            }
        }

        if let Some(old_args) = self.args.1.clone() {
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
                self.persist_config(args).await?;
                // self.update_config.send(args.to_vec())?;
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
        let primary = self.args.0.take();
        self.args.0 = Some(args);

        if let Err(e) = self.start_llamacpp_server().await {
            warn!("Failed to start server with new configuration: {}", e);
            self.args.0 = primary;
        } else {
            self.args.1 = primary;
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

    async fn persist_config(&self, args: &Vec<String>) -> Result<()> {
        let client = Client::new();

        client
            .post(format!("http://{:#?}/v1/config", self.supervisor_addr))
            .json(args)
            .send()
            .await?;

        Ok(())
    }
}

#[async_trait]
impl Service for ApplyingService {
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
