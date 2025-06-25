use anyhow::Result;
use anyhow::anyhow;
use cucumber::gherkin::Step;
use cucumber::then;
use reqwest::Response;

use crate::agent_response::AgentsResponse;
use crate::paddler_world::PaddlerWorld;

const MAX_ATTEMPTS: usize = 30;

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

// fn assert_fields(table_fields: Vec<Option<&String>>, peer_fields: Vec<String>) {
//     for (index, table_field) in table_fields.iter().enumerate() {
//         if let Some(field) = table_field {
//             assert_eq!(**field, peer_fields[index])
//         }
//     }
// }

#[then("balancer state is:")]
pub async fn then_balancer_state_is(world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    if let Some(table) = step.table.as_ref() {
        let headers = &table.rows[0];

        for row in table.rows.iter().skip(1) {
            for (col_idx, header) in headers.iter().enumerate() {
                match header.as_str() {
                    "agent" => {
                        world
                            .table_fields
                            .insert("agent".to_string(), row.get(col_idx).cloned());
                    }
                    "is_connect_error" => {
                        world
                            .table_fields
                            .insert("is_connect_error".to_string(), row.get(col_idx).cloned());
                    }
                    _ => continue,
                }
            }
        }
    }

    for field in &world.table_fields {
        let response = fetch_status(8095).await?;
        let upstream_peer_pool = response.json::<AgentsResponse>().await?;

        match field.key().as_str() {
            "agent" => {
                if let Some(value) = world.table_fields.get("agent") {
                    if let Some(agent_name) = value.value() {
                        let peer = upstream_peer_pool
                        .agents
                        .iter()
                        .find(|agent| agent.status.agent_name == *agent_name)
                        .ok_or_else(|| anyhow::anyhow!("not found in response"))?;

                        assert_eq!(peer.status.agent_name, *agent_name);
                    }
                }
            }
            "is_connect_error" => {
                if let Some(value) = world.table_fields.get("is_connect_error") {
                    if let Some(is_connect_error) = value.value() {
                        let expected_bool = is_connect_error.parse::<bool>()
                            .map_err(|_| anyhow!("Invalid bool string: {}", is_connect_error))?;
                    
                        let peer = upstream_peer_pool
                            .agents
                            .iter()
                            .find(|agent| agent.status.is_connect_error == Some(expected_bool))
                            .ok_or_else(|| anyhow!("not found in response"))?;

                        panic!("{:#?} | {:#?}", peer.status.is_connect_error, Some(expected_bool));
                    
                        assert_eq!(peer.status.is_connect_error, Some(expected_bool));
                    }
                }
            }
            _ => continue,
        }
    }

    Ok(())
}
