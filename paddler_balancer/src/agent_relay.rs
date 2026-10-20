use crate::agent_relay_conclusion::AgentRelayConclusion;
use crate::agent_relay_event::AgentRelayEvent;
use crate::agent_relay_step::AgentRelayStep;
use crate::agent_streaming_request::AgentStreamingRequest;
use crate::awaiting_agent_relay::AwaitingAgentRelay;
use crate::dispatched_agent_relay::DispatchedAgentRelay;

pub enum AgentRelay<TParams>
where
    TParams: AgentStreamingRequest,
{
    Awaiting(AwaitingAgentRelay<TParams>),
    Dispatched(DispatchedAgentRelay<TParams::Response>),
}

impl<TParams> AgentRelay<TParams>
where
    TParams: AgentStreamingRequest,
{
    pub async fn next(self) -> AgentRelayStep<TParams> {
        let dispatched_agent_relay = match self {
            Self::Awaiting(awaiting_agent_relay) => match awaiting_agent_relay.dispatch().await {
                Ok(dispatched_agent_relay) => dispatched_agent_relay,
                Err(relay_failure) => {
                    return AgentRelayStep::Concluded(AgentRelayConclusion::Failed(relay_failure));
                }
            },
            Self::Dispatched(dispatched_agent_relay) => dispatched_agent_relay,
        };

        match dispatched_agent_relay.agent_events.next().await {
            AgentRelayEvent::Failed(relay_failure) => {
                AgentRelayStep::Concluded(AgentRelayConclusion::Failed(relay_failure))
            }
            AgentRelayEvent::Final(final_result) => {
                AgentRelayStep::Concluded(AgentRelayConclusion::AgentConcluded {
                    final_result,
                    generated_by: dispatched_agent_relay.generated_by,
                })
            }
            AgentRelayEvent::Partial(partial_result) => AgentRelayStep::Partial {
                generated_by: dispatched_agent_relay.generated_by.clone(),
                partial_result,
                relay: Self::Dispatched(dispatched_agent_relay),
            },
        }
    }
}
