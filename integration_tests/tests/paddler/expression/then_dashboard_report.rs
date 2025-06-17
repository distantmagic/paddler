use core::panic;

use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::then;
use reqwest::Response;

use crate::paddler_world::PaddlerWorld;
use crate::upstream_peer_pool::UpstreamPeerPool;

async fn fetch_dashboard(balancer_port: u16) -> Result<Response> {
    let response = reqwest::get(format!("http://127.0.0.1:{balancer_port}/api/v1/agents")).await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Dashboard check failed: Expected status 200, got {}",
            response.status()
        ));
    }

    Ok(response)
}

#[then(expr = "dashboard report:")]
pub async fn then_dashboard_report(_world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    let response = fetch_dashboard(8095).await?.text().await?;
    let upstream_peer_pool: UpstreamPeerPool = serde_json::from_str(&response)?;

    if let Some(table) = step.table.as_ref() {
        for (i, row) in table.rows.iter().enumerate() {
            let peer = &upstream_peer_pool.agents[i];

            let agent_name = row[0].clone();
            let slots_idle = row[2].clone();
            let slots_processing = row[4].clone();
            let error = row[6].clone();

            assert_eq!(agent_name, peer.agent_name.clone().unwrap());
            assert_eq!(slots_idle, peer.slots_idle.to_string());
            assert_eq!(slots_processing, peer.slots_processing.to_string());
            assert_eq!(error, peer.error.clone().unwrap_or("none".to_string()));
        }
    };

    Ok(())
}
