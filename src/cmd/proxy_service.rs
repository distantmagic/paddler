use pingora_core::listeners::Listeners;
use pingora_core::services::listening::Service;
use pingora_core::upstreams::peer::HttpPeer;
use std::net::SocketAddr;

use crate::cmd::proxy_app::ProxyApp;

pub fn proxy_service(addr: SocketAddr, proxy_addr: SocketAddr) -> Service<ProxyApp> {
    let proxy_to = HttpPeer::new(
        proxy_addr,
        false,
        "".to_string(),
    );

    Service::with_listeners(
        "Proxy Service".to_string(),
        Listeners::tcp(&addr.to_string()),
        ProxyApp::new(proxy_to),
    )
}
