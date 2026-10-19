use std::num::NonZeroU16;

use paddler_messaging::inference_capability::InferenceCapability;

use crate::dispatch_requirement::DispatchRequirement;
use crate::slot_demand::SlotDemand;

pub const fn one_slot_of(inference_capability: InferenceCapability) -> DispatchRequirement {
    DispatchRequirement {
        inference_capability,
        slot_demand: SlotDemand::Exactly(NonZeroU16::MIN),
    }
}
