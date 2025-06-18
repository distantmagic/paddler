use anyhow::Result;
use cucumber::when;
use crate::paddler_world::PaddlerWorld;

#[when(expr = "llama.cpp server {string} stops running")]
pub async fn when_agent_detaches(
    world: &mut PaddlerWorld,
    llamacpp_name: String,
) -> Result<()> {
    if !world.llamas.instances.contains_key(&llamacpp_name) {
        return Err(anyhow::anyhow!(
            "Llama.cpp server {} is not running",
            llamacpp_name
        ));
    }

    world.agents.kill(llamacpp_name).await;

    Ok(())
}
