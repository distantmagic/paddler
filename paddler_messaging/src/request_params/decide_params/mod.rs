pub mod raw_decide_params;

use std::num::NonZeroU16;

use serde::Deserialize;
use serde::Serialize;

use crate::decision_question::DecisionQuestion;
use crate::decision_slots_minimum::DECISION_SLOTS_MINIMUM;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DecideParams {
    pub last_question: DecisionQuestion,
    pub leading_questions: Vec<DecisionQuestion>,
    pub state: String,
}

impl DecideParams {
    #[must_use]
    pub const fn required_slots(&self) -> NonZeroU16 {
        if self.leading_questions.is_empty() {
            NonZeroU16::MIN
        } else {
            DECISION_SLOTS_MINIMUM
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU16;

    use super::DecideParams;
    use crate::decision_question::DecisionQuestion;
    use crate::decision_slots_minimum::DECISION_SLOTS_MINIMUM;

    fn question(id: &str) -> DecisionQuestion {
        DecisionQuestion {
            id: id.to_owned(),
            instructions: "Was it paid?".to_owned(),
            options: vec!["no".to_owned(), "yes".to_owned()],
        }
    }

    #[test]
    fn a_single_question_decision_requires_only_its_state_slot() {
        assert_eq!(
            DecideParams {
                last_question: question("paid"),
                leading_questions: Vec::new(),
                state: "state".to_owned(),
            }
            .required_slots(),
            NonZeroU16::MIN
        );
    }

    #[test]
    fn a_decision_with_leading_questions_requires_its_state_and_question_lane_slots() {
        assert_eq!(
            DecideParams {
                last_question: question("paid"),
                leading_questions: vec![question("late")],
                state: "state".to_owned(),
            }
            .required_slots(),
            DECISION_SLOTS_MINIMUM
        );
    }
}
