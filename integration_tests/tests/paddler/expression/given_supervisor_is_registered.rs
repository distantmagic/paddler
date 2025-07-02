use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use cucumber::given;
use tokio::time::sleep;

use crate::paddler_world::PaddlerWorld;

const MAX_ATTEMPTS: usize = 30;

async fn do_check(world: &mut PaddlerWorld, supervisor_name: String) -> Result<()> {
    if !world.supervisors.instances.contains_key(&supervisor_name) {
        return Err(anyhow!(
            "Supervisor {supervisor_name} does not exist in the world"
        ));
    }

    let supervisors_response = world.balancer_management_client.fetch_supervisors().await?;
    let supervisor = supervisors_response
        .supervisors
        .iter()
        .find(|supervisor| supervisor.status.supervisor_name == supervisor_name)
        .ok_or_else(|| anyhow!("not found in response"))?;

    if let Some(error_value) = &supervisor.status.error {
        return Err(anyhow!("supervisor reported error: {error_value:?}"));
    }

    Ok(())
}

#[given(expr = "supervisor {string} is registered")]
pub async fn given_supervisor_is_registered(
    world: &mut PaddlerWorld,
    supervisor_name: String,
) -> Result<()> {
    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        sleep(Duration::from_millis(100)).await;

        if do_check(world, supervisor_name.clone()).await.is_ok() {
            return Ok(());
        }

        attempts += 1;
    }

    Err(anyhow!(
        "Supervisor '{supervisor_name}' is not registered after {MAX_ATTEMPTS} attempts"
    ))
}
