use anyhow::Result;
use anyhow::anyhow;
use cucumber::given;

use crate::paddler_world::PaddlerWorld;
use crate::spawn_agent_instance::spawn_agent_instance;

#[given(expr = "agent {string} is running \\(observes {string}\\)")]
pub async fn given_agent_is_running_observes_llama_server(
    world: &mut PaddlerWorld,
    agent_name: String,
    llamacpp_name: String,
) -> Result<()> {
    if world.agents.instances.contains_key(&agent_name) {
        return Err(anyhow!("Agent {agent_name} is already running"));
    }

    let local_llamacpp_port = world.llamas.llamacpp_port(&llamacpp_name)?;

    world.agents.instances.insert(
        agent_name.clone(),
        spawn_agent_instance(agent_name, local_llamacpp_port)?,
    );

    Ok(())
}
