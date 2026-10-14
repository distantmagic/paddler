use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use llama_cpp_bindings::token::LlamaToken;
use tokio::sync::mpsc;

use paddler_agent_decision::active_decision_request::ActiveDecisionRequest;
use paddler_agent_decision::decision_admission::DecisionAdmission;
use paddler_agent_decision::decision_capacity_demand::DecisionCapacityDemand;
use paddler_agent_decision::decision_question_row::DecisionQuestionRow;
use paddler_agent_decision::decision_token_layout::DecisionTokenLayout;
use paddler_agent_decision::prepared_decision_request::PreparedDecisionRequest;
use paddler_agent_status::slot_aggregated_status::SlotAggregatedStatus;
use paddler_agent_status::slot_guard::SlotGuard;
use paddler_messaging::decision_result::DecisionResult;

pub struct AdmittedDecision {
    pub active_decision_request: ActiveDecisionRequest,
    pub decision_result_rx: mpsc::UnboundedReceiver<DecisionResult>,
    pub decision_stop_tx: mpsc::UnboundedSender<()>,
}

impl AdmittedDecision {
    #[must_use]
    pub const fn capacity_demand(state: &[LlamaToken]) -> DecisionCapacityDemand {
        DecisionCapacityDemand {
            needs_question_lane: true,
            required_cells: state.len(),
        }
    }

    #[must_use]
    pub fn new(
        admission: DecisionAdmission,
        slot_aggregated_status: Arc<SlotAggregatedStatus>,
        state: Vec<LlamaToken>,
    ) -> Self {
        let (decision_result_tx, decision_result_rx) = mpsc::unbounded_channel();
        let (decision_stop_tx, decision_stop_rx) = mpsc::unbounded_channel();

        Self {
            active_decision_request: ActiveDecisionRequest::new(
                PreparedDecisionRequest {
                    capacity_demand: Self::capacity_demand(&state),
                    decision_result_tx,
                    decision_stop_rx,
                    layout: DecisionTokenLayout {
                        first_question: DecisionQuestionRow {
                            id: "question".to_owned(),
                            option_end_offsets: Vec::new(),
                            tokens: Vec::new(),
                        },
                        remaining_questions: VecDeque::new(),
                        state,
                    },
                    slot_guard: SlotGuard::new(slot_aggregated_status),
                    started_at: Instant::now(),
                },
                admission,
            ),
            decision_result_rx,
            decision_stop_tx,
        }
    }
}
