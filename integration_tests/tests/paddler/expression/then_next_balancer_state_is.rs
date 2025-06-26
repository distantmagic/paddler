use std::time::Duration;
use std::time::SystemTime;

use anyhow::Result;
<<<<<<< HEAD
use anyhow::anyhow;
=======
>>>>>>> 4e5a40cbc4fe507e6b9a5a9ac8695b7087d2ee11
use cucumber::gherkin::Step;
use cucumber::then;
use tokio::time::sleep;

use crate::agent_response::AgentsResponse;
<<<<<<< HEAD
=======
use crate::balancer_table::assert_balancer_table;
use crate::balancer_table::fetch_status;
>>>>>>> 4e5a40cbc4fe507e6b9a5a9ac8695b7087d2ee11
use crate::paddler_world::PaddlerWorld;

const MAX_ATTEMPTS: usize = 30;

<<<<<<< HEAD
async fn fetch_status(balancer_port: u16) -> Result<AgentsResponse> {
    let response = reqwest::get(format!("http://127.0.0.1:{balancer_port}/api/v1/agents")).await?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "Dashboard check failed: Expected status 200, got {}",
            response.status()
        ));
    }

    let agents_response = response.json::<AgentsResponse>().await?;
    Ok(agents_response)
}

fn compare_last_update(agents: AgentsResponse, last_update: SystemTime) -> bool {
    let mut agent_status_was_updated = false;

    for agent in agents.agents {
        agent_status_was_updated = agent.last_update > last_update
    }

    agent_status_was_updated
}

fn assert_fields(table_fields: Vec<Option<&String>>, peer_fields: Vec<String>) {
    for (index, table_field) in table_fields.iter().enumerate() {
        if let Some(field) = table_field {
            assert_eq!(**field, peer_fields[index])
        }
    }
=======
fn compare_last_update(agents: AgentsResponse, last_update: SystemTime) -> bool {
    for agent in agents.agents {
        if agent.last_update < last_update {
            return false;
        }
    }

    true
>>>>>>> 4e5a40cbc4fe507e6b9a5a9ac8695b7087d2ee11
}

#[then("next balancer state is:")]
pub async fn then_balancer_state_is(world: &mut PaddlerWorld, step: &Step) -> Result<()> {
<<<<<<< HEAD
    let last_update = world.last_update.expect("Last update does not exist");
=======
    let last_update = world
        .last_balancer_state_update
        .expect("Last update does not exist");
>>>>>>> 4e5a40cbc4fe507e6b9a5a9ac8695b7087d2ee11

    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        sleep(Duration::from_millis(100)).await;

<<<<<<< HEAD
        let agents_response = fetch_status(8095).await?;

        if compare_last_update(agents_response, last_update) {
            world.last_update = Some(SystemTime::now());
=======
        let response = fetch_status(8095).await?;
        let agents_response = response.json::<AgentsResponse>().await?;

        if compare_last_update(agents_response, last_update) {
            world.last_balancer_state_update = Some(SystemTime::now());
>>>>>>> 4e5a40cbc4fe507e6b9a5a9ac8695b7087d2ee11
            break;
        }

        attempts += 1;
    }

<<<<<<< HEAD
    let agents_response = fetch_status(8095).await?;

    if let Some(table) = step.table.as_ref() {
        let headers = &table.rows[0];

        for row in table.rows.iter().skip(1) {
            let agent_name = row
                .get(0)
                .ok_or_else(|| anyhow!("Missing agent name in table row"))?;

            let peer = agents_response
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
=======
    let response = fetch_status(8095).await?;
    let agents_response = response.json::<AgentsResponse>().await?;

    if let Some(table) = step.table.as_ref() {
        assert_balancer_table(table, &agents_response)?;
>>>>>>> 4e5a40cbc4fe507e6b9a5a9ac8695b7087d2ee11
    }

    Ok(())
}
