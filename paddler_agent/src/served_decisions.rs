use std::sync::Arc;

use paddler_agent_decision::decision_error::DecisionError;
use paddler_agent_decision::decision_request_preparer::DecisionRequestPreparer;
use paddler_agent_decision::reports_decision_outcome::ReportsDecisionOutcome as _;
use paddler_agent_runtime::agent_request::AgentRequest;
use paddler_agent_runtime::scheduler_handle::SchedulerHandle;
use paddler_agent_runtime::shuts_down_unless_shared::ShutsDownUnlessShared;
use paddler_messaging::decision_result::DecisionResult;
use paddler_messaging::inference_mode::InferenceMode;
use paddler_messaging::request_params::decide_params::DecideParams;

#[derive(Clone)]
pub enum ServedDecisions {
    Disabled,
    Enabled(Arc<SchedulerHandle<DecisionRequestPreparer>>),
}

impl ServedDecisions {
    pub fn forward(
        &self,
        agent_name: Option<&str>,
        request: AgentRequest<DecideParams, DecisionResult>,
    ) {
        match self {
            Self::Disabled => DecisionError::InferenceModeMismatch {
                serving_inference_mode: InferenceMode::TextGenerationAndEmbeddings,
            }
            .report(agent_name, &request.response_tx),
            Self::Enabled(scheduler_handle) => scheduler_handle.request_preparer.prepare(request),
        }
    }
}

impl ShutsDownUnlessShared for ServedDecisions {
    async fn shut_down_unless_shared(self) {
        match self {
            Self::Disabled => {}
            Self::Enabled(scheduler_handle) => scheduler_handle.shut_down_unless_shared().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use paddler_messaging::decision_result::DecisionResult;

    use super::ServedDecisions;
    use crate::decision_request_fixture::decision_request_fixture;

    #[test]
    fn an_agent_without_decisions_refuses_to_decide_naming_the_mode_it_serves() {
        let (decision_result_tx, mut decision_result_rx) = mpsc::unbounded_channel();

        ServedDecisions::Disabled
            .forward(Some("agent"), decision_request_fixture(decision_result_tx));

        assert_eq!(
            decision_result_rx.try_recv(),
            Ok(DecisionResult::InferenceModeMismatch(
                "agent: the agent serves TextGenerationAndEmbeddings, so it cannot decide"
                    .to_owned()
            ))
        );
    }
}
