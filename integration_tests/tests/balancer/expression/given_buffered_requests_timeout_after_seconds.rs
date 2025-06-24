use anyhow::Result;
use cucumber::given;

use crate::paddler_world::PaddlerWorld;

#[given(expr = "buffered requests timeout after {int} second(s)")]
pub async fn given_agent_is_attached(
    world: &mut PaddlerWorld,
    buffered_request_timeout: i64,
) -> Result<()> {
    world.buffered_request_timeout = Some(buffered_request_timeout);

    Ok(())
}
