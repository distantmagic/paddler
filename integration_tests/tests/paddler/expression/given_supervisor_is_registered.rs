use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use cucumber::given;

use crate::paddler_world::PaddlerWorld;
use crate::retry_until_success::retry_until_success;

const MAX_ATTEMPTS: usize = 30;

async fn do_check(world: &PaddlerWorld, supervisor_name: String) -> Result<()> {
    if !world.supervisors.instances.contains_key(&supervisor_name) {
        return Err(anyhow!(
            "Supervisor {supervisor_name} does not exist in the world"
        ));
    }

    let supervisors_response = world.balancer.management_client.fetch_supervisors().await?;

    supervisors_response
        .supervisors
        .iter()
        .find(|supervisor| supervisor.name == Some(supervisor_name.clone()))
        .ok_or_else(|| anyhow!("Not found in response"))?;

    Ok(())
}

#[given(expr = "supervisor {string} is registered")]
pub async fn given_supervisor_is_registered(
    world: &mut PaddlerWorld,
    supervisor_name: String,
) -> Result<()> {
    retry_until_success(
        || do_check(world, supervisor_name.clone()),
        MAX_ATTEMPTS,
        Duration::from_millis(100),
        format!("Supervisor '{supervisor_name}' is still not registered"),
    )
    .await
}
