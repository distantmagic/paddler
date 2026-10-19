use actix_web::http::StatusCode;

use paddler_messaging::decision_answer::DecisionAnswer;
use paddler_messaging::decision_result::DecisionResult;
use paddler_messaging::decision_summary::DecisionSummary;
use paddler_messaging::oversized_decision_details::OversizedDecisionDetails;

use crate::compatibility::typesafe_service::typesafe_decision_failure::TypeSafeDecisionFailure;
use crate::compatibility::upstream_failure::UpstreamFailure;

const fn failure(status_code: StatusCode, detail: String) -> DecisionEvent {
    DecisionEvent::Failed(TypeSafeDecisionFailure {
        detail,
        status_code,
    })
}

#[derive(Debug, PartialEq)]
pub enum DecisionEvent {
    Answered(DecisionAnswer),
    Failed(TypeSafeDecisionFailure),
    Finished(DecisionSummary),
}

impl From<DecisionResult> for DecisionEvent {
    fn from(decision_result: DecisionResult) -> Self {
        match decision_result {
            DecisionResult::QuestionAnswered(answer) => Self::Answered(answer),
            DecisionResult::Done(summary) => Self::Finished(summary),
            DecisionResult::RequestExceedsContext(OversizedDecisionDetails {
                context_size,
                required_tokens,
            }) => failure(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!(
                    "the request needs {required_tokens} tokens in the cache, more than the {context_size} the context holds"
                ),
            ),
            DecisionResult::InputTokenizationFailed(detail) => {
                failure(StatusCode::UNPROCESSABLE_ENTITY, detail)
            }
            DecisionResult::InferenceModeMismatch(detail)
            | DecisionResult::ModelNotLoaded(detail)
            | DecisionResult::SchedulerUnavailable(detail) => {
                failure(UpstreamFailure::Unavailable.status_code(), detail)
            }
            DecisionResult::AgentRuntimeFailed(detail)
            | DecisionResult::BatchAssemblyFailed(detail)
            | DecisionResult::DecodeFailed(detail)
            | DecisionResult::HiddenStateUnavailable(detail)
            | DecisionResult::KvCacheCopyFailed(detail)
            | DecisionResult::KvCacheRemovalFailed(detail) => {
                failure(UpstreamFailure::AgentFailed.status_code(), detail)
            }
            DecisionResult::StopRequested => Self::Failed(TypeSafeDecisionFailure::unfinished()),
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;

    use paddler_messaging::decision_answer::DecisionAnswer;
    use paddler_messaging::decision_result::DecisionResult;
    use paddler_messaging::decision_summary::DecisionSummary;
    use paddler_messaging::oversized_decision_details::OversizedDecisionDetails;

    use super::DecisionEvent;
    use crate::compatibility::typesafe_service::typesafe_decision_failure::TypeSafeDecisionFailure;

    fn failed(status_code: StatusCode, detail: &str) -> DecisionEvent {
        DecisionEvent::Failed(TypeSafeDecisionFailure {
            detail: detail.to_owned(),
            status_code,
        })
    }

    #[test]
    fn classifies_every_decision_result() {
        let answer = DecisionAnswer {
            id: "paid".to_owned(),
            probabilities: vec![0.25, 0.75],
        };
        let summary = DecisionSummary {
            input_tokens: 1,
            processing_milliseconds: 1,
        };

        for (decision_result, expected_event) in [
            (
                DecisionResult::QuestionAnswered(answer.clone()),
                DecisionEvent::Answered(answer),
            ),
            (
                DecisionResult::Done(summary.clone()),
                DecisionEvent::Finished(summary),
            ),
            (
                DecisionResult::RequestExceedsContext(OversizedDecisionDetails {
                    context_size: 128,
                    required_tokens: 410,
                }),
                failed(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "the request needs 410 tokens in the cache, more than the 128 the context holds",
                ),
            ),
            (
                DecisionResult::InputTokenizationFailed("nul byte".to_owned()),
                failed(StatusCode::UNPROCESSABLE_ENTITY, "nul byte"),
            ),
            (
                DecisionResult::InferenceModeMismatch("mode".to_owned()),
                failed(StatusCode::SERVICE_UNAVAILABLE, "mode"),
            ),
            (
                DecisionResult::ModelNotLoaded("no model".to_owned()),
                failed(StatusCode::SERVICE_UNAVAILABLE, "no model"),
            ),
            (
                DecisionResult::SchedulerUnavailable("stopped".to_owned()),
                failed(StatusCode::SERVICE_UNAVAILABLE, "stopped"),
            ),
            (
                DecisionResult::AgentRuntimeFailed("runtime".to_owned()),
                failed(StatusCode::INTERNAL_SERVER_ERROR, "runtime"),
            ),
            (
                DecisionResult::BatchAssemblyFailed("batch".to_owned()),
                failed(StatusCode::INTERNAL_SERVER_ERROR, "batch"),
            ),
            (
                DecisionResult::DecodeFailed("decode".to_owned()),
                failed(StatusCode::INTERNAL_SERVER_ERROR, "decode"),
            ),
            (
                DecisionResult::HiddenStateUnavailable("hidden".to_owned()),
                failed(StatusCode::INTERNAL_SERVER_ERROR, "hidden"),
            ),
            (
                DecisionResult::KvCacheCopyFailed("copy".to_owned()),
                failed(StatusCode::INTERNAL_SERVER_ERROR, "copy"),
            ),
            (
                DecisionResult::KvCacheRemovalFailed("removal".to_owned()),
                failed(StatusCode::INTERNAL_SERVER_ERROR, "removal"),
            ),
            (
                DecisionResult::StopRequested,
                failed(
                    StatusCode::BAD_GATEWAY,
                    "the agent stopped before it finished the decision",
                ),
            ),
        ] {
            assert_eq!(DecisionEvent::from(decision_result), expected_event);
        }
    }
}
