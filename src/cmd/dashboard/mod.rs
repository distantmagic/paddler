use std::net::SocketAddr;
use tokio::runtime::Runtime;

use crate::{
    balancer::upstream_peer_pool::UpstreamPeerPool, cmd::dashboard::app::App,
    errors::result::Result,
};

pub mod app;
pub mod ui;

pub async fn ratatui_main(management_addr: &SocketAddr) -> Result<()> {
    let agents = get_registered_agents(management_addr).await?;

    let terminal = ratatui::init();
    let app_result = App::new(agents)?.run(terminal)?;
    ratatui::restore();
    Ok(app_result)
}

pub fn handle(management_addr: &SocketAddr) -> Result<()> {
    Runtime::new()?.block_on(ratatui_main(management_addr))?;
    Ok(())
}

pub async fn get_registered_agents(management_addr: &SocketAddr) -> Result<UpstreamPeerPool> {
    let response_string = reqwest::get(format!("http://{}/api/v1/agents", management_addr.to_string().as_str()))
        .await?
        .text()
        .await?;

    let upstream_peer_pool: UpstreamPeerPool = serde_json::from_str(response_string.as_str())?;

    eprintln!("{:#?}", upstream_peer_pool.agents);

    // let addr_str = "127.0.0.1:8080";
    // let socket = addr_str.parse::<SocketAddr>().unwrap();

    // let upstream_peer_pool = UpstreamPeerPool {
    //     agents: RwLock::new(vec![
    //         UpstreamPeer {
    //             agent_id: String::from("123123123123123123123"),
    //             agent_name: None,
    //             error: None,
    //             external_llamacpp_addr: socket,
    //             is_authorized: true,
    //             last_update: SystemTime::now(),
    //             quarantined_until: Some(SystemTime::now()),
    //             slots_idle: 0,
    //             slots_processing: 0,
    //         },
    //         UpstreamPeer {
    //             agent_id: String::from("123123123123123123123"),
    //             agent_name: None,
    //             error: None,
    //             external_llamacpp_addr: socket,
    //             is_authorized: true,
    //             last_update: SystemTime::now(),
    //             quarantined_until: Some(SystemTime::now()),
    //             slots_idle: 0,
    //             slots_processing: 0,
    //         }
    //     ])
    // };

    Ok(upstream_peer_pool)
}
