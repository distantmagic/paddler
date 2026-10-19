use actix_web::HttpResponse;
use actix_web_lab::sse;
use futures::future::ready;
use futures::stream::Stream;
use futures::stream::StreamExt as _;
use futures::stream::iter;
use serde_json::Error as SerdeJsonError;
use serde_json::to_string;

use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_openai_translation::generation_event::GenerationEvent;
use paddler_openai_translation::responses_stream::ResponsesStream;
use paddler_openai_translation::responses_stream_event::ResponsesStreamEvent;

use crate::compatibility::agent_result_stream_event::AgentResultStreamEvent;
use crate::compatibility::openai_service::openai_sse_http_response::openai_sse_http_response;

fn responses_stream_events(
    responses_stream: &mut ResponsesStream,
    generated_token_event: AgentResultStreamEvent<GeneratedTokenResult>,
) -> Vec<ResponsesStreamEvent> {
    match generated_token_event {
        AgentResultStreamEvent::Result { result, .. } => {
            responses_stream.advance(GenerationEvent::from(result))
        }
        AgentResultStreamEvent::WireError(wire_error) => {
            responses_stream.fail(wire_error.description)
        }
    }
}

fn event_to_sse_data(event: &ResponsesStreamEvent) -> Result<sse::Data, SerdeJsonError> {
    to_string(event)
        .map(|serialized_event| sse::Data::new(serialized_event).event(event.event_name()))
}

pub fn responses_sse_response<TStream>(
    responses_stream: ResponsesStream,
    generated_tokens: TStream,
) -> HttpResponse
where
    TStream: Stream<Item = AgentResultStreamEvent<GeneratedTokenResult>> + 'static,
{
    openai_sse_http_response(
        generated_tokens
            .scan(
                responses_stream,
                |responses_stream, generated_token_event| {
                    ready(Some(responses_stream_events(
                        responses_stream,
                        generated_token_event,
                    )))
                },
            )
            .flat_map(iter)
            .map(|event| event_to_sse_data(&event).map(sse::Event::Data)),
    )
}
