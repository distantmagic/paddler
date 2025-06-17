use anyhow::Result;
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
        .ok_or_else(|| anyhow::anyhow!("Llama.cpp server {} not found", llamacpp_name))?;

    if !llamacpp.accepted_request(&request_name).await? {
        return Err(anyhow::anyhow!(
            "Request '{}' did not land in Llama.cpp server '{}'",
            request_name,
            llamacpp_name
        ));
    }

    Ok(())
}
