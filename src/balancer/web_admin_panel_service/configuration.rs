use std::net::SocketAddr;

#[derive(Clone)]
pub struct Configuration {
    pub addr: SocketAddr,
    pub inference_addr: SocketAddr,
    pub management_addr: SocketAddr,
}
