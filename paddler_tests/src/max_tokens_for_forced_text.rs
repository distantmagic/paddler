use std::num::NonZeroU32;

use anyhow::Result;

pub fn max_tokens_for_forced_text(forced_text: &str) -> Result<NonZeroU32> {
    Ok(NonZeroU32::try_from(u32::try_from(forced_text.len())?)?)
}
