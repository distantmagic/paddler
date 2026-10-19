use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;
use paddler_messaging::inference_capability::InferenceCapability;

use crate::dispatch_requirement::DispatchRequirement;
use crate::slot_demand::SlotDemand;

pub const DECISION_WITH_LEADING_QUESTIONS_REQUIREMENT: DispatchRequirement = DispatchRequirement {
    inference_capability: InferenceCapability::Decisions,
    slot_demand: SlotDemand::Exactly(DECISION_SLOTS_MINIMUM),
};
