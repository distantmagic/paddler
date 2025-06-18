use core::panic;
use std::time::Duration;

use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::then;
use reqwest::Response;
use tokio::time::sleep;

use crate::agent_status::AgentStatusResponse;
use crate::paddler_world::PaddlerWorld;

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
    let upstream_peer_pool: AgentStatusResponse = serde_json::from_str(&response)?;

    if let Some(table) = step.table.as_ref() {
        for (i, row) in table.rows.iter().skip(1).enumerate() {
            // panic!("{:#?}", table.rows.iter().skip(1).enumerate());

            // panic!("{:#?}", &upstream_peer_pool.agents);

            let peer = &upstream_peer_pool.agents[i];

            let agent_name = row[0].clone();
            let slots_idle = row[1].clone();
            let slots_processing = row[2].clone();

            assert_eq!(agent_name, peer.agent_name.clone());
            assert_eq!(slots_idle, peer.slots_idle.to_string());
            assert_eq!(slots_processing, peer.slots_processing.to_string());
        }
    };

    Ok(())
}
