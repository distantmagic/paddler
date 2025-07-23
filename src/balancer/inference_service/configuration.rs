use std::net::SocketAddr;
use std::time::Duration;

#[derive(Clone)]
pub struct Configuration {
    pub addr: SocketAddr,
    pub cors_allowed_hosts: Vec<String>,
    pub inference_token_timeout: Duration,
}
