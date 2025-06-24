use std::time::Duration;

use anyhow::Result;
use cucumber::given;
use serde::Deserialize;
use tokio::time::sleep;

use crate::paddler_world::PaddlerWorld;

const MAX_ATTEMPTS: usize = 30;

#[derive(Deserialize)]
struct AgentStatus {
    agent_name: String,
    error: Option<String>,
}

#[derive(Deserialize)]
struct Agent {
    status: AgentStatus,
}

#[derive(Deserialize)]
struct AgentsResponse {
    agents: Vec<Agent>,
}

async fn do_check(world: &mut PaddlerWorld, agent_name: String) -> Result<()> {
    if !world.agents.instances.contains_key(&agent_name) {
        return Err(anyhow::anyhow!(
            "Agent {agent_name} does not exist in the world"
        ));
    }

    let response = reqwest::get("http://127.0.0.1:8095/api/v1/agents").await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to get agent status: {}",
            response.status()
        ));
    }

    let agents_response = response.json::<AgentsResponse>().await?;
    let agent = agents_response
        .agents
        .iter()
        .find(|agent| agent.status.agent_name == agent_name)
        .ok_or_else(|| anyhow::anyhow!("not found in response"))?;

    if let Some(error_value) = &agent.status.error {
        return Err(anyhow::anyhow!("agent reported error: {:?}", error_value));
    }

    Ok(())
}

#[given(expr = "agent {string} is healthy")]
pub async fn given_agent_is_healthy(world: &mut PaddlerWorld, agent_name: String) -> Result<()> {
    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        sleep(Duration::from_millis(100)).await;

        if do_check(world, agent_name.clone()).await.is_ok() {
            return Ok(());
        }

        attempts += 1;
    }

    Err(anyhow::anyhow!(
        "Agent '{}' is not healthy after {} attempts",
        agent_name,
        MAX_ATTEMPTS
    ))
}
