use std::time::SystemTime;

use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::then;

use crate::agent_response::AgentsResponse;
use crate::balancer_table::assert_balancer_table;
use crate::balancer_table::fetch_status;
use crate::paddler_world::PaddlerWorld;

<<<<<<< HEAD
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

fn get_agent_last_update(agents_response: &AgentsResponse) -> SystemTime {
    let mut last_update = SystemTime::now();

    for agent in &agents_response.agents {
        if agent.last_update > last_update {
            last_update = agent.last_update;
        }
    }

    last_update
}

=======
>>>>>>> 4e5a40cbc4fe507e6b9a5a9ac8695b7087d2ee11
#[then("balancer state is:")]
pub async fn then_balancer_state_is(world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    let response = fetch_status(8095).await?;
    let agents_response = response.json::<AgentsResponse>().await?;

<<<<<<< HEAD
    let last_agents_update = get_agent_last_update(&agents_response);

    world.last_update = Some(last_agents_update);

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
    world.last_balancer_state_update = Some(SystemTime::now());

    if let Some(table) = step.table.as_ref() {
        assert_balancer_table(table, &agents_response)?;
>>>>>>> 4e5a40cbc4fe507e6b9a5a9ac8695b7087d2ee11
    }

    Ok(())
}
