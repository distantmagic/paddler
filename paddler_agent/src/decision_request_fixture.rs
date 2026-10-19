use std::num::NonZeroU16;
use std::sync::Arc;

use tokio::sync::mpsc;

use paddler_agent_runtime::agent_request::AgentRequest;
use paddler_agent_status::slot_aggregated_status::SlotAggregatedStatus;
use paddler_agent_status::slot_guard::SlotGuard;
use paddler_messaging::decision_question::DecisionQuestion;
use paddler_messaging::decision_result::DecisionResult;
use paddler_messaging::request_params::decide_params::DecideParams;

pub fn decision_request_fixture(
    decision_result_tx: mpsc::UnboundedSender<DecisionResult>,
) -> AgentRequest<DecideParams, DecisionResult> {
    let (_decision_stop_tx, decision_stop_rx) = mpsc::unbounded_channel();

    AgentRequest {
        params: DecideParams {
            last_question: DecisionQuestion {
                id: "paid".to_owned(),
                instructions: "Was it paid?".to_owned(),
                options: vec!["no".to_owned(), "yes".to_owned()],
            },
            leading_questions: Vec::new(),
            state: "state".to_owned(),
        },
        response_tx: decision_result_tx,
        slot_guard: SlotGuard::new(Arc::new(SlotAggregatedStatus::new(1)), NonZeroU16::MIN),
        stop_rx: decision_stop_rx,
    }
}
