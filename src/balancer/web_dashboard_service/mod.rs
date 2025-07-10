pub mod configuration;
pub mod http_route;

use std::sync::Arc;

use actix_web::web::Data;
use actix_web::App;
use actix_web::HttpServer;
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::broadcast;

// use crate::balancer::upstream_peer_pool::UpstreamPeerPool;
use crate::balancer::web_dashboard_service::configuration::Configuration as WebDashboardServiceConfiguration;
use crate::service::Service;

pub struct WebDashboardService {
    configuration: WebDashboardServiceConfiguration,
    // upstream_peers: Arc<UpstreamPeerPool>,
}

impl WebDashboardService {
    pub fn new(
        configuration: WebDashboardServiceConfiguration,
        // upstream_peers: Arc<UpstreamPeerPool>,
    ) -> Self {
        WebDashboardService {
            configuration,
            // upstream_peers,
        }
    }
}

#[async_trait]
impl Service for WebDashboardService {
    async fn run(&mut self, mut _shutdown: broadcast::Receiver<()>) -> Result<()> {
        // let upstream_peers: Data<UpstreamPeerPool> = self.upstream_peers.clone().into();
        let configuration: Data<WebDashboardServiceConfiguration> =
            Data::new(self.configuration.clone());

        Ok(HttpServer::new(move || {
            App::new()
                .app_data(configuration.clone())
                // .app_data(upstream_peers.clone())
                .configure(http_route::dashboard::register)
                .configure(http_route::static_files::register)
        })
        .bind(self.configuration.addr)
        .expect("Unable to bind server to address")
        .run()
        .await?)
    }
}
