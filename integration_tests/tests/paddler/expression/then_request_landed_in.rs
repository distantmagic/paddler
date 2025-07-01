use anyhow::Result;
use anyhow::anyhow;
use cucumber::then;

use crate::paddler_world::PaddlerWorld;

#[then(expr = "{string} request landed in {string}")]
pub async fn then_request_landed_in(
    world: &mut PaddlerWorld,
    request_name: String,
    llamacpp_name: String,
) -> Result<()> {
    let llamacpp = world
        .llamas
        .instances
        .get(&llamacpp_name)
        .ok_or_else(|| anyhow!("Llama.cpp server {llamacpp_name} not found"))?;

    let accepted_result = llamacpp.accepted_request(&request_name).await?;

    if !accepted_result.accepted {
        return Err(anyhow!(
            "Request '{request_name}' did not land in Llama.cpp server '{llamacpp_name}'.\nLogs: {}",
            accepted_result.contents,
        ));
    }

    Ok(())
}
