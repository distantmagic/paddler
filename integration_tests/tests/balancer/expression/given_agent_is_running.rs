use std::process::Stdio;

use anyhow::Result;
use cucumber::given;
use tokio::process::Command;

use crate::balancer_world::BalancerWorld;

#[given(expr = "agent {string} is running \\(observes {string}\\)")]
pub async fn given_agent_is_attached(
    world: &mut BalancerWorld,
    agent_name: String,
    llamacpp_name: String,
) -> Result<()> {
    if world.agents.contains_key(&agent_name) {
        return Err(anyhow::anyhow!("Agent {} is already running", agent_name));
    }

    let local_llamacpp_port = world.get_llamacpp_port(&llamacpp_name)?;

    world.agents.insert(
        agent_name.clone(),
        Command::new("../target/debug/paddler")
            .arg("agent")
            .arg(format!("--name={agent_name}"))
            .arg(format!(
                "--local-llamacpp-addr=127.0.0.1:{local_llamacpp_port}"
            ))
            .arg("--management-addr=127.0.0.1:8095")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
    );

    Ok(())
}
