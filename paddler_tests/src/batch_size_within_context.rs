use std::num::NonZeroU32;

use anyhow::Result;

use paddler_inference_parameters::batch_size::BatchSize;

pub fn batch_size_within_context(context_size: NonZeroU32) -> Result<BatchSize> {
    let batch_tokens = context_size.get().min(BatchSize::DEFAULT.tokens().get());

    Ok(BatchSize::try_from(batch_tokens)?)
}
