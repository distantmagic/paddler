use std::num::NonZeroU16;

use paddler_agent_runtime::sequence_id_guard::SequenceIdGuard;
use paddler_agent_runtime::sequence_id_pool::SequenceIdPool;

use crate::decision_admission::DecisionAdmission;
use crate::question_lane::QuestionLane;

pub struct DecisionCapacityLedger {
    sequence_id_pool: SequenceIdPool,
}

impl DecisionCapacityLedger {
    #[must_use]
    pub fn new(sequences: u16) -> Self {
        Self {
            sequence_id_pool: SequenceIdPool::new(sequences),
        }
    }

    #[must_use]
    pub fn admit(&self, reserved_slots: NonZeroU16) -> Option<DecisionAdmission> {
        let state_sequence = SequenceIdGuard::acquire(&self.sequence_id_pool)?;
        let question_lane = if reserved_slots > NonZeroU16::MIN {
            QuestionLane::Reserved(SequenceIdGuard::acquire(&self.sequence_id_pool)?)
        } else {
            QuestionLane::Released
        };

        Some(DecisionAdmission {
            question_lane,
            state_sequence,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;

    use super::DecisionCapacityLedger;

    #[test]
    fn a_single_question_decision_takes_only_its_state_sequence() {
        let ledger = DecisionCapacityLedger::new(2);
        let _first_admission = ledger
            .admit(NonZeroU16::MIN)
            .expect("one free sequence fits a single question");

        assert!(ledger.admit(NonZeroU16::MIN).is_some());
    }

    #[test]
    fn a_multi_question_decision_waits_for_a_free_question_lane() {
        let ledger = DecisionCapacityLedger::new(1);

        assert!(ledger.admit(DECISION_SLOTS_MINIMUM).is_none());
    }

    #[test]
    fn a_decision_runs_once_the_sequences_of_a_finished_one_are_returned() {
        let ledger = DecisionCapacityLedger::new(2);
        let first_admission = ledger.admit(DECISION_SLOTS_MINIMUM);

        assert!(ledger.admit(NonZeroU16::MIN).is_none());

        drop(first_admission);

        assert!(ledger.admit(NonZeroU16::MIN).is_some());
    }
}
