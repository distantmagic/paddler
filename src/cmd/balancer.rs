use actix_web::{App, HttpServer};
use pingora::prelude::*;
use pingora::upstreams::peer::HttpPeer;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::task;

use crate::balancer::http_route;
use crate::cmd::proxy::ProxyApp;
use crate::errors::result::Result;

pub async fn handle(management_addr: &SocketAddr, reverseproxy_addr: &SocketAddr) -> Result<()> {
    let mut join_set = task::JoinSet::new();

    let management_server =
        HttpServer::new(move || App::new().configure(http_route::receive_status_update::register))
            .bind(management_addr)?
            .run();

    join_set.spawn(management_server);
    join_set.spawn(async move {
        let peer = HttpPeer::new(
            "127.0.0.1:8080".parse::<SocketAddr>().unwrap(),
            false,
            "".to_string(),
        );
        let proxy_app = ProxyApp::new(peer);

        Ok(())
    });

    join_set.join_all().await;

    Ok(())
}
