use actix_web::{App, HttpServer};
use pingora::prelude::*;
use std::net::SocketAddr;
use tokio::task;

use crate::balancer::http_route;
use crate::errors::result::Result;

pub async fn handle(management_addr: &SocketAddr, reverseproxy_addr: &SocketAddr) -> Result<()> {
    let mut join_set = task::JoinSet::new();

    let management_server =
        HttpServer::new(move || App::new().configure(http_route::receive_status_update::register))
            .bind(management_addr)?
            .run();

    join_set.spawn(management_server);
    join_set.spawn(async move {
        let mut my_server = Server::new(None).unwrap();
        my_server.bootstrap();
        my_server.run_forever();

        Ok(())
    });

    join_set.join_all().await;

    Ok(())
}
