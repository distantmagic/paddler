use std::time::SystemTime;

use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::then;

use crate::agent_response::AgentsResponse;
use crate::balancer_table::assert_balancer_table;
use crate::balancer_table::fetch_status;
use crate::paddler_world::PaddlerWorld;

fn get_agent_last_update(agents_response: &AgentsResponse) -> SystemTime {
    let mut last_update = SystemTime::now();

    for agent in &agents_response.agents {
        if agent.last_update > last_update {
            last_update = agent.last_update;
        }
    }

    last_update
}

#[then("balancer state is:")]
pub async fn then_balancer_state_is(world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    let response = fetch_status(8095).await?;
    let agents_response = response.json::<AgentsResponse>().await?;

    let last_agents_update = get_agent_last_update(&agents_response);

    world.last_update = Some(last_agents_update);

    if let Some(table) = step.table.as_ref() {
        assert_balancer_table(table, &agents_response)?;
    }

    Ok(())
}
