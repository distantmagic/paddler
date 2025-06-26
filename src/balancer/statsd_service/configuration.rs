use std::net::SocketAddr;
use std::time::Duration;

pub struct Configuration {
    pub statsd_addr: SocketAddr,
    pub statsd_prefix: String,
    pub statsd_reporting_interval: Duration,
}
