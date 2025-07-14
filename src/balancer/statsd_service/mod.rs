pub mod configuration;

use std::net::UdpSocket;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use cadence::BufferedUdpMetricSink;
use cadence::Gauged;
use cadence::StatsdClient;
use log::error;
use tokio::sync::broadcast;
use tokio::time::interval;
use tokio::time::MissedTickBehavior;

use crate::balancer::agent_controller_pool::AgentControllerPool;
use crate::balancer::statsd_service::configuration::Configuration as StatsdServiceConfiguration;
use crate::service::Service;

pub struct StatsdService {
    agent_controller_pool: Arc<AgentControllerPool>,
    configuration: StatsdServiceConfiguration,
}

impl StatsdService {
    pub fn new(
        agent_controller_pool: Arc<AgentControllerPool>,
        configuration: StatsdServiceConfiguration,
    ) -> Result<Self> {
        Ok(StatsdService {
            agent_controller_pool,
            configuration,
        })
    }

    async fn report_metrics(&self, client: &StatsdClient) -> Result<()> {
        let (slots_idle, slots_processing) = self.agent_controller_pool.total_slots()?;
        let requests_buffered = self.agent_controller_pool.total_buffered_requests();

        client.gauge("slots_idle", slots_idle as u64)?;
        client.gauge("slots_processing", slots_processing as u64)?;
        client.gauge("requests_buffered", requests_buffered as u64)?;
        client.flush()?;

        Ok(())
    }
}

#[async_trait]
impl Service for StatsdService {
    fn name(&self) -> &'static str {
        "balancer::statsd_service"
    }

    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let statsd_sink_socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
        let statsd_sink =
            BufferedUdpMetricSink::from(self.configuration.statsd_addr, statsd_sink_socket)
                .expect("Failed to create statsd sink");

        let client =
            StatsdClient::builder(&self.configuration.statsd_prefix.to_owned(), statsd_sink)
                .with_error_handler(|err| error!("Statsd error: {err}"))
                .build();

        let mut ticker = interval(self.configuration.statsd_reporting_interval);

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    return Ok(());
                },
                _ = ticker.tick() => {
                    if let Err(err) = self.report_metrics(&client).await {
                        error!("Failed to report metrics: {err}");
                    }
                }
            }
        }
    }
}
