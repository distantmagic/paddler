use std::time::Duration;

use anyhow::Result;
use cucumber::when;
use tokio::time::sleep;

use crate::agent_response::AgentsResponse;
use crate::paddler_world::PaddlerWorld;

const MAX_ATTEMPTS: usize = 30;

async fn do_check(balancer_port: u16) -> Result<()> {
    let response = reqwest::get(format!("http://127.0.0.1:{balancer_port}/api/v1/agents")).await?;
    let agents_response = response.json::<AgentsResponse>().await?;

    let _ = agents_response
        .agents
        .iter()
        .find(|agent| agent.status.is_connect_error == Some(true))
        .ok_or_else(|| anyhow::anyhow!("not found in response"))?;

    Ok(())
}

#[when(expr = "llama.cpp server {string} stops running")]
pub async fn when_agent_detaches(world: &mut PaddlerWorld, llamacpp_name: String) -> Result<()> {
    if !world.llamas.instances.contains_key(&llamacpp_name) {
        return Err(anyhow::anyhow!(
            "Llama.cpp server {} is not running",
            llamacpp_name
        ));
    }

    let llamacpp_port = world.llamas.llamacpp_port(&llamacpp_name)?;

    world.llamas.kill(&llamacpp_name).await?;

    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        sleep(Duration::from_millis(100)).await;

        if do_check(8095).await.is_ok() {
            return Ok(());
        } else {
            eprintln!(
                "Attempt {}: llama.cpp at port {} is still alive.",
                attempts + 1,
                llamacpp_port
            );
        }

        attempts += 1;
    }

    Err(anyhow::anyhow!(
        "Llama.cpp server at port {} is still running after {} attempts",
        llamacpp_port,
        MAX_ATTEMPTS
    ))
}
