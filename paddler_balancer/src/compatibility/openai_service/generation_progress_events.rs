use async_stream::stream;
use futures::stream::Stream;

use paddler_messaging::generated_token_final_result::GeneratedTokenFinalResult;
use paddler_messaging::generated_token_result::GeneratedTokenResult;
use paddler_openai_translation::generation_progress::GenerationProgress;

use crate::agent_relay::AgentRelay;
use crate::agent_relay_conclusion::AgentRelayConclusion;
use crate::agent_relay_event::AgentRelayEvent;
use crate::agent_relay_step::AgentRelayStep;
use crate::agent_streaming_request::AgentStreamingRequest;

pub fn generation_progress_events<TParams>(
    agent_relay: AgentRelay<TParams>,
) -> impl Stream<Item = AgentRelayEvent<GenerationProgress, GeneratedTokenFinalResult>>
where
    TParams: AgentStreamingRequest<Response = GeneratedTokenResult>,
{
    stream! {
        let settlement = agent_relay.settlement();
        let mut agent_relay = agent_relay;

        loop {
            match agent_relay.next().await {
                AgentRelayStep::Concluded(AgentRelayConclusion::AgentConcluded {
                    final_result,
                    ..
                }) => {
                    yield AgentRelayEvent::Final(final_result);
                    settlement.cancelled().await;

                    break;
                }
                AgentRelayStep::Concluded(AgentRelayConclusion::Failed(relay_failure)) => {
                    yield AgentRelayEvent::Failed(relay_failure);
                    settlement.cancelled().await;

                    break;
                }
                AgentRelayStep::Partial {
                    partial_result,
                    relay,
                    ..
                } => match GenerationProgress::from(partial_result) {
                    failed_generation @ GenerationProgress::Failed(_) => {
                        yield AgentRelayEvent::Partial(failed_generation);
                        drop(relay);
                        settlement.cancelled().await;

                        break;
                    }
                    generation_progress => {
                        yield AgentRelayEvent::Partial(generation_progress);

                        agent_relay = relay;
                    }
                },
            }
        }
    }
}
