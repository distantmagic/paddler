use std::pin::pin;

use actix_web::HttpResponse;
use futures::stream::Stream;
use futures::stream::StreamExt as _;

use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_openai_translation::completes_generation::CompletesGeneration;
use paddler_openai_translation::generated_output::GeneratedOutput;
use paddler_openai_translation::generation_event::GenerationEvent;

use crate::compatibility::agent_result_stream_event::AgentResultStreamEvent;
use crate::compatibility::openai_service::openai_failure::OpenAIFailure;

pub async fn openai_http_response<TCompletesGeneration, TStream>(
    completes_generation: &TCompletesGeneration,
    generated_tokens: TStream,
) -> HttpResponse
where
    TCompletesGeneration: CompletesGeneration,
    TStream: Stream<Item = AgentResultStreamEvent<GeneratedTokenResult>>,
{
    let mut generated_tokens = pin!(generated_tokens);
    let mut generated_output = GeneratedOutput::default();

    while let Some(generated_token_event) = generated_tokens.next().await {
        match generated_token_event {
            AgentResultStreamEvent::Result { request_id, result } => {
                match GenerationEvent::from(result) {
                    GenerationEvent::Produced(generated_output_part) => {
                        generated_output.append(generated_output_part);
                    }
                    GenerationEvent::ToolCallTokenProduced => {}
                    GenerationEvent::Finished(generation_summary) => {
                        return HttpResponse::Ok().json(completes_generation.complete(
                            &request_id,
                            generated_output,
                            &generation_summary,
                        ));
                    }
                    GenerationEvent::Failed(generation_failure) => {
                        return OpenAIFailure::Generation(generation_failure).into_http_response();
                    }
                }
            }
            AgentResultStreamEvent::WireError(wire_error) => {
                return OpenAIFailure::Wire(wire_error).into_http_response();
            }
        }
    }

    OpenAIFailure::Unfinished.into_http_response()
}

#[cfg(test)]
mod tests {
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;
    use futures::stream::empty;
    use serde_json::Value;
    use serde_json::from_slice;
    use serde_json::json;

    use paddler_messaging::generated_token_result::GeneratedTokenResult;
    use paddler_openai_translation::chat_completion_header::ChatCompletionHeader;

    use super::openai_http_response;
    use crate::compatibility::agent_result_stream_event::AgentResultStreamEvent;

    #[actix_web::test]
    async fn a_generation_stream_ending_without_a_result_answers_a_bad_gateway() {
        let response = openai_http_response(
            &ChatCompletionHeader {
                created: 1,
                model: "test-model".to_owned(),
            },
            empty::<AgentResultStreamEvent<GeneratedTokenResult>>(),
        )
        .await;

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        assert_eq!(
            from_slice::<Value>(&to_bytes(response.into_body()).await.unwrap()).unwrap(),
            json!({
                "error": {
                    "code": null,
                    "message": "the agent stopped before it finished the generation",
                    "param": null,
                    "type": "server_error"
                }
            })
        );
    }
}
