use std::net::SocketAddr;
use std::time::Duration;

#[derive(Clone)]
pub struct TemplateData {
    pub buffered_request_timeout: Duration,
    pub inference_addr: SocketAddr,
    pub management_addr: SocketAddr,
    pub max_buffered_requests: i32,
    pub statsd_addr: Option<SocketAddr>,
    pub statsd_prefix: String,
    pub statsd_reporting_interval: Duration,
}
