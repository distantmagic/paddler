use pingora::server::configuration::Opt;
use pingora::server::Server;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::balancer::management_service::ManagementService;
use crate::balancer::proxy_service::ProxyService;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
use crate::errors::result::Result;

pub fn handle(management_addr: &SocketAddr, reverseproxy_addr: &SocketAddr) -> Result<()> {
    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    let upstream_peer_pool = Arc::new(UpstreamPeerPool::new());

    let management_service = ManagementService::new(*management_addr, upstream_peer_pool.clone());
    let mut proxy_service =
        pingora_proxy::http_proxy_service(&pingora_server.configuration, ProxyService {});

    proxy_service.add_tcp(&reverseproxy_addr.clone().to_string());

    pingora_server.bootstrap();
    pingora_server.add_service(management_service);
    pingora_server.add_service(proxy_service);

    pingora_server.run_forever();
}
