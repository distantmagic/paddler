use std::time::Duration;

use anyhow::Result;
use cucumber::given;
use serde_json::Value;
use tokio::time::sleep;

use crate::balancer_world::BalancerWorld;

const MAX_ATTEMPTS: usize = 3;

async fn do_check(world: &mut BalancerWorld, agent_name: String) -> Result<()> {
    if !world.agents.contains_key(&agent_name) {
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

    let body = response.json::<Value>().await?;

    let agents = body
        .get("agents")
        .and_then(|agents| agents.as_array())
        .ok_or_else(|| anyhow::anyhow!("Invalid response format: 'agents' not found"))?;

    let agent_status = agents
        .iter()
        .find(|agent| {
            agent
                .get("agent_name")
                .and_then(|agent_name| agent_name.as_str())
                == Some(&agent_name)
        })
        .ok_or_else(|| anyhow::anyhow!("not found in response"))?;

    let error = agent_status.get("error");

    if let Some(error_value) = error {
        if error_value.is_null() {
            return Ok(());
        }

        return Err(anyhow::anyhow!("agent reported error: {:?}", error_value));
    }

    Ok(())
}

#[given(expr = "agent {string} is healthy")]
pub async fn given_agent_is_healthy(world: &mut BalancerWorld, agent_name: String) -> Result<()> {
    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        sleep(Duration::from_secs(1)).await;

        match do_check(world, agent_name.clone()).await {
            Ok(_) => return Ok(()),
            Err(err) => eprintln!(
                "Attempt {}: Agent '{}' is not healthy - {}",
                attempts + 1,
                agent_name,
                err
            ),
        }

        attempts += 1;
    }

    Err(anyhow::anyhow!(
        "Agent '{}' is not healthy after {} attempts",
        agent_name,
        MAX_ATTEMPTS
    ))
}
