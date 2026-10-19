use tokio_util::task::TaskTracker;

use crate::pipeline_request::PipelineRequest;
use crate::served_model::ServedModel;

pub enum ModelServing {
    Idle,
    Serving(Box<ServedModel>),
}

impl ModelServing {
    pub fn forward(&self, agent_name: Option<&str>, request: PipelineRequest) {
        match self {
            Self::Idle => request.reject_because_no_model_is_loaded(agent_name),
            Self::Serving(served_model) => served_model.forward(agent_name, request),
        }
    }

    pub async fn shut_down(self) {
        match self {
            Self::Idle => {}
            Self::Serving(served_model) => served_model.shut_down().await,
        }
    }

    pub fn shut_down_in_background(self, task_tracker: &TaskTracker) {
        task_tracker.spawn(self.shut_down());
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use paddler_messaging::decision_result::DecisionResult;

    use super::ModelServing;
    use crate::decision_request_fixture::decision_request_fixture;
    use crate::pipeline_request::PipelineRequest;

    #[test]
    fn an_agent_serving_no_model_refuses_requests_because_no_model_is_loaded() {
        let (decision_result_tx, mut decision_result_rx) = mpsc::unbounded_channel();

        ModelServing::Idle.forward(
            Some("agent"),
            PipelineRequest::Decide(decision_request_fixture(decision_result_tx)),
        );

        assert_eq!(
            decision_result_rx.try_recv(),
            Ok(DecisionResult::ModelNotLoaded(
                "agent: no model is loaded".to_owned()
            ))
        );
    }
}
