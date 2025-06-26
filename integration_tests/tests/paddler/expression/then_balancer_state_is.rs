use std::time::SystemTime;

use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::then;

use crate::agent_response::AgentsResponse;
use crate::balancer_table::assert_balancer_table;
use crate::balancer_table::fetch_status;
use crate::paddler_world::PaddlerWorld;

#[then("balancer state is:")]
pub async fn then_balancer_state_is(world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    let response = fetch_status(8095).await?;
    let agents_response = response.json::<AgentsResponse>().await?;

    world.last_balancer_state_update = Some(SystemTime::now());

    if let Some(table) = step.table.as_ref() {
        assert_balancer_table(table, &agents_response)?;
    }

    Ok(())
}
