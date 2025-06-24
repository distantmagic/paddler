use anyhow::Result;
use cucumber::given;

use crate::paddler_world::PaddlerWorld;

#[given(expr = "request buffering is disabled")]
pub async fn given_agent_is_attached(world: &mut PaddlerWorld) -> Result<()> {
    world.max_buffered_requests = Some(0);

    Ok(())
}
