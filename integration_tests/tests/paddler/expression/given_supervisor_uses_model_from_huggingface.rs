use anyhow::Result;
use anyhow::anyhow;
use cucumber::given;

use crate::paddler_world::PaddlerWorld;

#[given(
    expr = "supervisor {string} uses model from Hugging Face \\(repo: {string}, weights: {string}\\)"
)]
pub async fn given_supervisor_uses_model_from_huggingface(
    world: &mut PaddlerWorld,
    supervisor_name: String,
    repo: String,
    weights: String,
) -> Result<()> {
    if !world.supervisors.instances.contains_key(&supervisor_name) {
        return Err(anyhow!(
            "Supervisor {supervisor_name} does not exist in the world"
        ));
    }

    Ok(())
}
