use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use cucumber::given;

use crate::paddler_world::PaddlerWorld;
use crate::retry_until_success::retry_until_success;

const MAX_ATTEMPTS: usize = 30;

async fn do_check(world: &PaddlerWorld, agent_name: String) -> Result<()> {
    if !world.agents.instances.contains_key(&agent_name) {
        return Err(anyhow!("Agent {agent_name} does not exist in the world"));
    }

    let agents_response = world.balancer.management_client.fetch_agents().await?;
    let agent = agents_response
        .agents
        .iter()
        .find(|agent| agent.status.agent_name == agent_name)
        .ok_or_else(|| anyhow!("Agent not registered yet"))?;

    if let Some(error_value) = &agent.status.error {
        return Err(anyhow!("Agent reported error: {error_value:?}"));
    }

    Ok(())
}

#[given(expr = "agent {string} is registered")]
pub async fn given_agent_is_registered(world: &mut PaddlerWorld, agent_name: String) -> Result<()> {
    retry_until_success(
        || do_check(world, agent_name.clone()),
        MAX_ATTEMPTS,
        Duration::from_millis(100),
        format!("Agent '{agent_name}' is still not registered"),
    )
    .await
}
