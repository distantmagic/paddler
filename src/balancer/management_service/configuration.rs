use std::net::SocketAddr;

#[derive(Clone)]
pub struct Configuration {
    pub addr: SocketAddr,
    pub cors_allowed_hosts: Vec<String>,
    pub fleet_management_enable: bool,
    pub metrics_endpoint_enable: bool,
}
