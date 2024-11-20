use pingora::{
    proxy::http_proxy_service,
    server::{configuration::Opt, Server},
};
use std::{net::SocketAddr, sync::Arc};

use crate::balancer::management_service::ManagementService;
use crate::balancer::proxy_service::ProxyService;
use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
use crate::errors::result::Result;

pub fn handle(
    management_addr: &SocketAddr,
    management_dashboard_enable: bool,
    reverseproxy_addr: &SocketAddr,
    rewrite_host_header: bool,
) -> Result<()> {
    let mut pingora_server = Server::new(Opt {
        upgrade: false,
        daemon: false,
        nocapture: false,
        test: false,
        conf: None,
    })?;

    let upstream_peer_pool = Arc::new(UpstreamPeerPool::new());

    let management_service = ManagementService::new(
        *management_addr,
        management_dashboard_enable,
        upstream_peer_pool.clone(),
    );
    let mut proxy_service = http_proxy_service(
        &pingora_server.configuration,
        ProxyService::new(rewrite_host_header, upstream_peer_pool.clone()),
    );

    proxy_service.add_tcp(&reverseproxy_addr.clone().to_string());

    pingora_server.bootstrap();
    pingora_server.add_service(management_service);
    pingora_server.add_service(proxy_service);

    pingora_server.run_forever();
}
