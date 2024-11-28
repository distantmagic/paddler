use std::net::SocketAddr;
use tokio::runtime::Runtime;

use crate::{
    cmd::dashboard::app::App,
    errors::result::Result,
    balancer::upstream_peer_pool::UpstreamPeerPool,
};

pub mod app;
pub mod render;
pub mod ui;

pub async fn ratatui_main(management_addr: &SocketAddr) -> Result<()> {
    let agents = get_registered_agents(management_addr).await?;

    let terminal = ratatui::init();
    let app_result = App::new(agents)?.run(terminal);
    ratatui::restore();
    app_result
}

pub fn handle(management_addr: &SocketAddr) -> Result<()> {
    Runtime::new()?.block_on(ratatui_main(management_addr))?;
    Ok(())
}

pub async fn get_registered_agents(management_addr: &SocketAddr) -> Result<UpstreamPeerPool> {
    let response_string = reqwest::get(management_addr.to_string().as_str()).await?.text().await?;
    let deserialized: UpstreamPeerPool = serde_json::from_str(response_string.as_str())?;

    Ok(deserialized)
}
