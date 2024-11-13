use actix_web::{App, HttpServer};

use crate::balancer::http_route;
use crate::errors::result::Result;

pub async fn handle(_management_addr: &url::Url, _reverseproxy_addr: &url::Url) -> Result<()> {
    Ok(
        HttpServer::new(move || App::new().configure(http_route::receive_status_update::register))
            .bind("127.0.0.1:8095")?
            .run()
            .await?,
    )
}
