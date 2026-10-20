use std::pin::pin;

use actix_web::HttpResponse;
use futures::stream::Stream;
use futures::stream::StreamExt as _;

use paddler_messaging::decision_result::DecisionResult;
use paddler_typesafe_translation::system_one_response_layout::SystemOneResponseLayout;

use crate::compatibility::agent_result_stream_event::AgentResultStreamEvent;
use crate::compatibility::typesafe_service::decision_event::DecisionEvent;
use crate::compatibility::typesafe_service::typesafe_decision_failure::TypeSafeDecisionFailure;

pub async fn system_one_http_response<TStream>(
    response_layout: &SystemOneResponseLayout,
    decision_results: TStream,
) -> HttpResponse
where
    TStream: Stream<Item = AgentResultStreamEvent<DecisionResult>>,
{
    let mut decision_results = pin!(decision_results);
    let mut answers = Vec::new();

    while let Some(decision_result_event) = decision_results.next().await {
        match decision_result_event {
            AgentResultStreamEvent::Result { result, .. } => match DecisionEvent::from(result) {
                DecisionEvent::Answered(answer) => answers.push(answer),
                DecisionEvent::Failed(typesafe_decision_failure) => {
                    return typesafe_decision_failure.into_http_response();
                }
                DecisionEvent::Finished(summary) => {
                    return match response_layout.respond(&answers, &summary) {
                        Ok(system_one_response) => HttpResponse::Ok().json(system_one_response),
                        Err(translation_error) => {
                            TypeSafeDecisionFailure::from(translation_error).into_http_response()
                        }
                    };
                }
            },
            AgentResultStreamEvent::WireError(wire_error) => {
                return TypeSafeDecisionFailure::from(wire_error).into_http_response();
            }
        }
    }

    TypeSafeDecisionFailure::unfinished().into_http_response()
}

#[cfg(test)]
mod tests {
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use futures::stream::empty;
    use serde_json::Value;
    use serde_json::from_slice;
    use serde_json::from_value;
    use serde_json::json;

    use paddler_messaging::decision_result::DecisionResult;
    use paddler_typesafe_translation::system_one_request::SystemOneRequest;

    use super::system_one_http_response;
    use crate::compatibility::agent_result_stream_event::AgentResultStreamEvent;

    #[actix_web::test]
    async fn a_decision_stream_ending_without_a_result_answers_a_bad_gateway() {
        let response = system_one_http_response(
            &from_value::<SystemOneRequest>(
                json!({"state": "state", "questions": {"paid": {"type": "noul"}}}),
            )
            .unwrap()
            .translate()
            .unwrap()
            .response_layout,
            empty::<AgentResultStreamEvent<DecisionResult>>(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert_eq!(
            from_slice::<Value>(&to_bytes(response.into_body()).await.unwrap()).unwrap(),
            json!({"detail": "the agent stopped before it finished the decision"})
        );
    }
}
