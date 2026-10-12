use anyhow::Result;

use paddler_agent_pointer_head::pointer_head::PointerHead;

use crate::kev_0_8b_target_path::kev_0_8b_target_path;

pub fn kev_0_8b_pointer_head() -> Result<PointerHead> {
    Ok(PointerHead::load(&kev_0_8b_target_path(
        "pointer_head.gguf",
    ))?)
}
