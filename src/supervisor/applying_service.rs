use std::process::{Child, Command, Stdio};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use pingora::{server::ShutdownWatch, services::Service};
use tokio::{
    sync::broadcast::Receiver,
    time::{interval, Duration, MissedTickBehavior},
};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::errors::result::Result;

pub struct ApplyingService {
    args: (Option<Vec<String>>, Option<Vec<String>>),
    monitoring_interval: Duration,
    llama_process: Option<Child>,
    update_llamacpp: Receiver<Vec<String>>,
}

impl ApplyingService {
    pub fn new(
        args: Vec<String>,
        monitoring_interval: Duration,
        update_llamacpp: Receiver<Vec<String>>,
    ) -> Result<Self> {
        Ok(ApplyingService {
            args: (Some(args), None),
            monitoring_interval,
            llama_process: None,
            update_llamacpp,
        })
    }

    async fn start_llamacpp_server(&mut self) -> Result<()> {
        if let Some(args) = self.args.0.clone() {
            if self.spawn_llama_process(&args).is_ok() {
                return Ok(());
            }
        }
        
        if let Some(old_args) = self.args.1.clone() {
            self.spawn_llama_process(&old_args)?;
        }
        
        Ok(())
    }

    fn spawn_llama_process(&mut self, args: &Vec<String>) -> Result<()> {
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
                        } else {
                            info!("Llamacpp server restarted.");
                        }
                    }
                },
                args = self.update_llamacpp.recv() => {
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
