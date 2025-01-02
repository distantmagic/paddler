use std::{
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

use crate::errors::result::Result;

use super::llamacpp_configuration::LlamacppConfiguration;

pub struct ApplyingService {
    llamacpp_options: LlamacppConfiguration,
    monitoring_interval: Duration,
    llama_process: Option<Child>,
    update_llamacpp: Receiver<LlamacppConfiguration>,
}

impl ApplyingService {
    pub fn new(
        llamacpp_options: LlamacppConfiguration,
        monitoring_interval: Duration,
        update_llamacpp: Receiver<LlamacppConfiguration>,
    ) -> Result<Self> {
        Ok(ApplyingService {
            llamacpp_options,
            monitoring_interval,
            llama_process: None,
            update_llamacpp,
        })
    }

    async fn start_llamacpp_server(&mut self) -> Result<()> {
        let mut cmd = Command::new(self.llamacpp_options.clone().binary);

        LlamacppConfiguration::is_a_gguf_file(self.llamacpp_options.clone().model)?;

        let port = self.llamacpp_options.clone().get_port();
        let host = self.llamacpp_options.clone().get_host();

        cmd.args(&[
            "-m",
            &self.llamacpp_options.model,
            "--host",
            &host,
            "--port",
            &port,
            "-t",
            &self.llamacpp_options.threads.to_string(),
            "-n",
            &self.llamacpp_options.predict.to_string(),
            "--temp",
            &self.llamacpp_options.temperature.to_string(),
            "-c",
            &self.llamacpp_options.ctx_size.to_string(),
            "--props",
            "--slots",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null());

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

    fn set_options(&mut self, options: LlamacppConfiguration) {
        self.llamacpp_options = options;
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
                        info!("Llamacpp server fell off. Restarting server");
                    }
                },
                input = self.update_llamacpp.recv() => {
                    match input {
                        Ok(options) => {
                            if let Some(process) = &mut self.llama_process {
                                let _ = process.kill();
                                let _ = process.wait();
                            }

                            self.set_options(options);
                            match self.start_llamacpp_server().await {
                                Ok(_) => {info!("Configuration was updated. Restarting server");},
                                Err(e) => {warn!("Failed to start llama server. Changes were not applied {}", e);}
                            }
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
