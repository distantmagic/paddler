use std::num::NonZeroU16;

use paddler_messaging::agent_runtime_status::AgentRuntimeStatus;
use paddler_messaging::inference_capability::InferenceCapability;

use crate::slot_demand::SlotDemand;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DispatchRequirement {
    pub inference_capability: InferenceCapability,
    pub slot_demand: SlotDemand,
}

impl DispatchRequirement {
    #[must_use]
    pub fn reservable_slots(
        &self,
        agent_runtime_status: &AgentRuntimeStatus,
        slots_processing: u64,
    ) -> Option<NonZeroU16> {
        self.slot_demand.reservable_from(
            agent_runtime_status
                .slots_serving(self.inference_capability)
                .saturating_sub(slots_processing),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use paddler_messaging::agent_runtime_status::AgentRuntimeStatus;
    use paddler_messaging::inference_capability::InferenceCapability;
    use paddler_messaging::inference_mode::InferenceMode;

    use super::DispatchRequirement;
    use crate::slot_demand::SlotDemand;

    const SERVING_FOUR_SLOTS: AgentRuntimeStatus = AgentRuntimeStatus::Serving {
        inference_mode: InferenceMode::TextGenerationAndEmbeddings,
        slots_total: 4,
    };

    fn requirement(inference_capability: InferenceCapability) -> DispatchRequirement {
        DispatchRequirement {
            inference_capability,
            slot_demand: SlotDemand::UpTo(8),
        }
    }

    #[test]
    fn reserves_the_slots_left_after_the_ones_in_use() {
        assert_eq!(
            requirement(InferenceCapability::Embeddings).reservable_slots(&SERVING_FOUR_SLOTS, 1),
            NonZeroU16::new(3)
        );
    }

    #[test]
    fn reserves_nothing_from_an_agent_using_more_slots_than_it_serves() {
        assert_eq!(
            requirement(InferenceCapability::Embeddings).reservable_slots(&SERVING_FOUR_SLOTS, 5),
            None
        );
    }

    #[test]
    fn reserves_nothing_for_a_capability_the_agent_does_not_serve() {
        assert_eq!(
            requirement(InferenceCapability::Decisions).reservable_slots(&SERVING_FOUR_SLOTS, 0),
            None
        );
    }
}
