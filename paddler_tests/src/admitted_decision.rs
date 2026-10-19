use std::num::NonZeroU16;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::mpsc;

use paddler_agent_decision::active_decision_request::ActiveDecisionRequest;
use paddler_agent_decision::decision_admission::DecisionAdmission;
use paddler_agent_decision::decision_token_layout::DecisionTokenLayout;
use paddler_agent_decision::prepared_decision_request::PreparedDecisionRequest;
use paddler_agent_decision::question_lane::QuestionLane;
use paddler_agent_status::slot_aggregated_status::SlotAggregatedStatus;
use paddler_agent_status::slot_guard::SlotGuard;
use paddler_messaging::decision_result::DecisionResult;
use paddler_messaging::decision_slots_minimum::DECISION_SLOTS_MINIMUM;

pub struct AdmittedDecision {
    pub active_decision_request: ActiveDecisionRequest,
    pub decision_result_rx: mpsc::UnboundedReceiver<DecisionResult>,
    pub decision_stop_tx: mpsc::UnboundedSender<()>,
}

impl AdmittedDecision {
    #[must_use]
    pub fn new(
        admission: DecisionAdmission,
        slot_aggregated_status: Arc<SlotAggregatedStatus>,
        layout: DecisionTokenLayout,
    ) -> Self {
        let (decision_result_tx, decision_result_rx) = mpsc::unbounded_channel();
        let (decision_stop_tx, decision_stop_rx) = mpsc::unbounded_channel();
        let reserved_slots = match admission.question_lane {
            QuestionLane::Reserved(_) => DECISION_SLOTS_MINIMUM,
            QuestionLane::Released => NonZeroU16::MIN,
        };

        Self {
            active_decision_request: ActiveDecisionRequest::new(
                PreparedDecisionRequest {
                    decision_result_tx,
                    decision_stop_rx,
                    layout,
                    slot_guard: SlotGuard::new(slot_aggregated_status, reserved_slots),
                    started_at: Instant::now(),
                },
                admission,
            ),
            decision_result_rx,
            decision_stop_tx,
        }
    }
}
