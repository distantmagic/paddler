use std::time::Duration;
use std::time::SystemTime;

use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::then;
use tokio::time::sleep;

use crate::agent_response::AgentsResponse;
use crate::balancer_table::assert_balancer_table;
use crate::balancer_table::fetch_status;
use crate::paddler_world::PaddlerWorld;

const MAX_ATTEMPTS: usize = 30;

fn compare_last_update(agents: AgentsResponse, last_update: SystemTime) -> bool {
    for agent in agents.agents {
        if agent.last_update > last_update {
            return true;
        }
    }
    false
}

#[then("next balancer state is:")]
pub async fn then_balancer_state_is(world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    let last_update = world.last_update.expect("Last update does not exist");

    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        sleep(Duration::from_millis(100)).await;

        let response = fetch_status(8095).await?;
        let agents_response = response.json::<AgentsResponse>().await?;

        if compare_last_update(agents_response, last_update) {
            world.last_update = Some(SystemTime::now());
            break;
        }

        attempts += 1;
    }

    let response = fetch_status(8095).await?;
    let agents_response = response.json::<AgentsResponse>().await?;

    if let Some(table) = step.table.as_ref() {
        assert_balancer_table(table, &agents_response)?;
    }

    Ok(())
}
