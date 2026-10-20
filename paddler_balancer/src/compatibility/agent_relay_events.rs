use async_stream::stream;
use futures::stream::Stream;

use paddler_messaging::streamable_result::StreamableResult;

use crate::agent_relay::AgentRelay;
use crate::agent_relay_conclusion::AgentRelayConclusion;
use crate::agent_relay_event::AgentRelayEvent;
use crate::agent_relay_step::AgentRelayStep;
use crate::agent_streaming_request::AgentStreamingRequest;

pub fn agent_relay_events<TParams>(
    agent_relay: AgentRelay<TParams>,
) -> impl Stream<
    Item = AgentRelayEvent<
        <TParams::Response as StreamableResult>::Partial,
        <TParams::Response as StreamableResult>::Final,
    >,
>
where
    TParams: AgentStreamingRequest,
{
    stream! {
        let mut agent_relay = agent_relay;

        loop {
            match agent_relay.next().await {
                AgentRelayStep::Concluded(AgentRelayConclusion::AgentConcluded {
                    final_result,
                    ..
                }) => {
                    yield AgentRelayEvent::Final(final_result);

                    break;
                }
                AgentRelayStep::Concluded(AgentRelayConclusion::Failed(relay_failure)) => {
                    yield AgentRelayEvent::Failed(relay_failure);

                    break;
                }
                AgentRelayStep::Partial {
                    partial_result,
                    relay,
                    ..
                } => {
                    yield AgentRelayEvent::Partial(partial_result);

                    agent_relay = relay;
                }
            }
        }
    }
}
