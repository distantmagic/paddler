use anyhow::Result;
use cucumber::given;
use tempfile::NamedTempFile;

use crate::fleet_management_state::FleetManagementState;
use crate::paddler_world::PaddlerWorld;

#[given(expr = "fleet management is enabled")]
pub async fn given_fleet_management_is_enabled(world: &mut PaddlerWorld) -> Result<()> {
    world.fleet_management_state = Some(FleetManagementState {
        fleet_database_file: NamedTempFile::new()?,
    });

    Ok(())
}
