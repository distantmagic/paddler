use actix_web::{App, HttpServer};
use futures::future;
use std::net::SocketAddr;

use crate::balancer::http_route;
use crate::errors::result::Result;

pub async fn handle(management_addr: &SocketAddr, reverseproxy_addr: &SocketAddr) -> Result<()> {
    let management_server =
        HttpServer::new(move || App::new().configure(http_route::receive_status_update::register))
            .bind(management_addr)?
            .run();

    let reverseproxy_server =
        HttpServer::new(move || App::new().configure(http_route::receive_status_update::register))
            .bind(reverseproxy_addr)?
            .run();

    future::try_join(management_server, reverseproxy_server).await?;

    Ok(())
}
