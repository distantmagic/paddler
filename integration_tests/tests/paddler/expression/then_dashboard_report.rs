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

fn assert_fields(table_fields: Vec<Option<&String>>, peer_fields: Vec<String>) {
    for (index, table_field) in table_fields.iter().enumerate() {
        if let Some(field) = table_field {
            assert_eq!(**field, peer_fields[index])
        }
    }
}

#[then(expr = "dashboard report:")]
pub async fn then_dashboard_report(_world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    let response = fetch_dashboard(8095).await?.text().await?;
    let upstream_peer_pool: AgentStatusResponse = serde_json::from_str(&response)?;

    if let Some(table) = step.table.as_ref() {
        let headers = &table.rows[0];

        for (i, row) in table.rows.iter().skip(1).enumerate() {
            let peer = &upstream_peer_pool.agents[i];

            let mut table_fields = Vec::new();
            let mut peer_fields = Vec::new();

            for (col_idx, header) in headers.iter().enumerate() {
                match header.as_str() {
                    "agent" => {
                        table_fields.push(row.get(col_idx));
                        peer_fields.push(peer.agent_name.clone());
                    }
                    "slots_idle" => {
                        table_fields.push(row.get(col_idx));
                        peer_fields.push(peer.slots_idle.to_string());
                    }
                    "slots_processing" => {
                        table_fields.push(row.get(col_idx));
                        peer_fields.push(peer.slots_processing.to_string());
                    }
                    "error" => {
                        table_fields.push(row.get(col_idx));
                        peer_fields.push(
                            peer.error
                                .as_ref()
                                .map(|b| b.to_string())
                                .unwrap_or("None".to_string()),
                        );
                    }
                    _ => continue,
                }
            }
            // panic!("{:#?}, {:#?}", table_fields, peer_fields);

            // sleep(Duration::from_secs(2)).await;

            assert_fields(table_fields, peer_fields);
        }
    }

    Ok(())
}
