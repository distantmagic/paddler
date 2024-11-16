use pingora::server::configuration::Opt;
use pingora::server::Server;
use std::net::SocketAddr;

use crate::balancer::management_service::ManagementService;
use crate::errors::result::Result;

pub fn handle(management_addr: &SocketAddr, reverseproxy_addr: &SocketAddr) -> Result<()> {
    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    pingora_server.bootstrap();
    pingora_server.add_service(ManagementService::new(*management_addr));
    pingora_server.run_forever();
}
