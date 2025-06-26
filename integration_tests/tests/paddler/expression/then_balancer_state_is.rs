use anyhow::Result;
use anyhow::anyhow;
use cucumber::gherkin::Step;
use cucumber::then;
use reqwest::Response;

use crate::agent_response::AgentsResponse;
use crate::paddler_world::PaddlerWorld;

async fn fetch_status(balancer_port: u16) -> Result<Response> {
    let response = reqwest::get(format!("http://127.0.0.1:{balancer_port}/api/v1/agents")).await?;
    if !response.status().is_success() {
        return Err(anyhow!(
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

#[then("balancer state is:")]
pub async fn then_balancer_state_is(_world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    let response = fetch_status(8095).await?.text().await?;
    let upstream_peer_pool: AgentsResponse = serde_json::from_str(&response)?;

    if let Some(table) = step.table.as_ref() {
        let headers = &table.rows[0];

        for row in table.rows.iter().skip(1) {
            let agent_name = row
                .get(0)
                .ok_or_else(|| anyhow!("Missing agent name in table row"))?;

            let peer = upstream_peer_pool
                .agents
                .iter()
                .find(|p| &p.status.agent_name == agent_name)
                .ok_or_else(|| anyhow!("Agent {} not found in response", agent_name))?;

            let mut table_fields = Vec::new();
            let mut peer_fields = Vec::new();

            for (col_idx, header) in headers.iter().enumerate() {
                match header.as_str() {
                    "agent" => {
                        table_fields.push(row.get(col_idx));
                        peer_fields.push(peer.status.agent_name.clone());
                    }
                    "is_connect_error" => {
                        table_fields.push(row.get(col_idx));
                        peer_fields.push(
                            peer.status
                                .is_connect_error
                                .map(|v| v.to_string())
                                .unwrap_or("None".to_string()),
                        );
                    }
                    _ => continue,
                }
            }

            assert_fields(table_fields, peer_fields);
        }
    }

    Ok(())
}
