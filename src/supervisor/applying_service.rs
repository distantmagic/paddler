use std::process::Output;

use async_process::Command;
use async_trait::async_trait;
use log::{debug, error};
use pingora::{server::ShutdownWatch, services::Service};
use tokio::time::{interval, Duration, MissedTickBehavior};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{errors::result::Result, llamacpp::llamacpp_client::LlamacppClient};

pub struct ApplyingService {
    llamacpp_client: LlamacppClient,
    llama_server_path: String,
    model_path: String,
    monitoring_interval: Duration,
}

impl ApplyingService {
    pub fn new(
        llamacpp_client: LlamacppClient,
        llama_server_path: String,
        model_path: String,
        monitoring_interval: Duration,
    ) -> Result<Self> {
        // let agent_id = Uuid::new_v4();

        Ok(ApplyingService {
            llamacpp_client,
            llama_server_path,
            model_path,
            monitoring_interval,
        })
    }

    async fn start_llamacpp_server(&self) -> Output {
        Command::new(self.llama_server_path.to_owned())
            .args(&[
                "-m",
                self.model_path.as_str(),
                "--port",
                self.llamacpp_client
                    .clone()
                    .get_address()
                    .to_string()
                    .split(':')
                    .nth(1)
                    .unwrap(),
                "--slots",
            ])
            .output()
            .await
            .unwrap()
    }

    async fn server_is_running(&self) -> bool {
        unsafe {
            let out = Command::new("lsof")
                .arg("-i")
                .arg(format!(
                    ":{}",
                    self.llamacpp_client
                        .clone()
                        .get_address()
                        .to_string()
                        .split(':')
                        .nth(1)
                        .unwrap()
                ))
                .output()
                .await
                .unwrap_unchecked();

            if out.stdout.is_empty() {
                return false;
            }

            return true;
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

        // self.start_llamacpp_server().await;

        Command::new(self.llama_server_path.to_owned())
            .args(&[
                "-m",
                self.model_path.as_str(),
                "--port",
                self.llamacpp_client
                    .clone()
                    .get_address()
                    .to_string()
                    .split(':')
                    .nth(1)
                    .unwrap(),
                "--slots",
            ])
            .output()
            .await
            .unwrap();

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down reporting service");
                    return;
                },
                // _ = ticker.tick() => {
                //     let a = self.server_is_running().await;

                //     eprintln!("{}", a);
                //     if !self.server_is_running().await {
                //         let _out = self.start_llamacpp_server().await;
                //     };
                // }
                // llama-server -m /usr/local/phi-1_5-Q8_0.gguf -c 2048 -ngl 2000 -np 4 -cp --port 8088 --slots
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
