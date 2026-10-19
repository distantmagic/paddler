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
