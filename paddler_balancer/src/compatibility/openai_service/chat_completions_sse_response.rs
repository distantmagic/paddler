use actix_web::HttpResponse;
use actix_web_lab::sse;
use futures::future::ready;
use futures::stream::Stream;
use futures::stream::StreamExt as _;
use futures::stream::iter;
use futures::stream::once;
use serde_json::Error as SerdeJsonError;

use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_openai_translation::chat_completion_stream::ChatCompletionStream;
use paddler_openai_translation::generation_event::GenerationEvent;

use crate::compatibility::agent_result_stream_event::AgentResultStreamEvent;
use crate::compatibility::openai_service::openai_failure::OpenAIFailure;
use crate::compatibility::openai_service::openai_sse_http_response::openai_sse_http_response;

fn openai_failure_data(openai_failure: OpenAIFailure) -> Vec<Result<sse::Data, SerdeJsonError>> {
    vec![sse::Data::new_json(openai_failure.into_error_body())]
}

fn chat_completion_sse_data(
    chat_completion_stream: &mut ChatCompletionStream,
    generated_token_event: AgentResultStreamEvent<GeneratedTokenResult>,
) -> Vec<Result<sse::Data, SerdeJsonError>> {
    match generated_token_event {
        AgentResultStreamEvent::Result { request_id, result } => {
            match chat_completion_stream.advance(&request_id, GenerationEvent::from(result)) {
                Ok(chat_completion_chunks) => chat_completion_chunks
                    .into_iter()
                    .map(sse::Data::new_json)
                    .collect(),
                Err(generation_failure) => {
                    openai_failure_data(OpenAIFailure::Generation(generation_failure))
                }
            }
        }
        AgentResultStreamEvent::WireError(wire_error) => {
            openai_failure_data(OpenAIFailure::Wire(wire_error))
        }
    }
}

pub fn chat_completions_sse_response<TStream>(
    chat_completion_stream: ChatCompletionStream,
    generated_tokens: TStream,
) -> HttpResponse
where
    TStream: Stream<Item = AgentResultStreamEvent<GeneratedTokenResult>> + 'static,
{
    openai_sse_http_response(
        generated_tokens
            .scan(
                chat_completion_stream,
                |chat_completion_stream, generated_token_event| {
                    ready(Some(chat_completion_sse_data(
                        chat_completion_stream,
                        generated_token_event,
                    )))
                },
            )
            .flat_map(iter)
            .chain(once(ready(Ok(sse::Data::new("[DONE]")))))
            .map(|sse_data| sse_data.map(sse::Event::Data)),
    )
}
