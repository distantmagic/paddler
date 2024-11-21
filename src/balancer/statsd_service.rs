use async_trait::async_trait;
use cadence::{BufferedUdpMetricSink, Gauged, StatsdClient};
use log::{debug, error};
use pingora::{server::ShutdownWatch, services::Service};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::Arc,
};
use tokio::time::{interval, Duration, MissedTickBehavior};

#[cfg(unix)]
use pingora::server::ListenFds;

use crate::{balancer::upstream_peer_pool::UpstreamPeerPool, errors::result::Result};

pub struct StatsdService {
    statsd_addr: SocketAddr,
    statsd_prefix: String,
    statsd_reporting_interval: Duration,
    upstream_peer_pool: Arc<UpstreamPeerPool>,
}

impl StatsdService {
    pub fn new(
        statsd_addr: SocketAddr,
        statsd_prefix: String,
        statsd_reporting_interval: Duration,
        upstream_peer_pool: Arc<UpstreamPeerPool>,
    ) -> Result<Self> {
        Ok(StatsdService {
            statsd_addr,
            statsd_prefix,
            statsd_reporting_interval,
            upstream_peer_pool,
        })
    }

    async fn report_metrics(&self, client: &StatsdClient) -> Result<()> {
        let (slots_idle, slots_processing) = self.upstream_peer_pool.total_slots()?;

        client.gauge("slots_idle", slots_idle as u64)?;
        client.gauge("slots_processing", slots_processing as u64)?;
        client.flush()?;

        Ok(())
    }
}

#[async_trait]
impl Service for StatsdService {
    async fn start_service(
        &mut self,
        #[cfg(unix)] _fds: Option<ListenFds>,
        mut shutdown: ShutdownWatch,
    ) {
        let statsd_sink_socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
        let statsd_sink = BufferedUdpMetricSink::from(self.statsd_addr, statsd_sink_socket)
            .expect("Failed to create statsd sink");

        let client = StatsdClient::builder(&self.statsd_prefix.to_owned(), statsd_sink)
            .with_error_handler(|err| error!("Statsd error: {}", err))
            .build();

        let mut ticker = interval(self.statsd_reporting_interval);

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    debug!("Shutting down monitoring service");
                    return;
                },
                _ = ticker.tick() => {
                    if let Err(err) = self.report_metrics(&client).await {
                        error!("Failed to fetch status: {}", err);
                    }
                }
            }
        }
    }

    fn name(&self) -> &str {
        "statsd"
    }

    fn threads(&self) -> Option<usize> {
        Some(1)
    }
}
