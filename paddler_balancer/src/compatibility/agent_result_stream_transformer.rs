use std::marker::PhantomData;

use async_trait::async_trait;

use paddler_messaging::inference_client::message::Message as OutgoingMessage;
use paddler_messaging::inference_client::response::Response as OutgoingResponse;
use paddler_messaging::jsonrpc::error_envelope::ErrorEnvelope;
use paddler_messaging::jsonrpc::response_envelope::ResponseEnvelope;

use crate::agent_relay_error::AgentRelayError;
use crate::chunk_forwarding_session_controller::transforms_outgoing_message::TransformsOutgoingMessage;
use crate::compatibility::agent_result_stream_event::AgentResultStreamEvent;

pub struct AgentResultStreamTransformer<TResult> {
    result: PhantomData<fn() -> TResult>,
}

impl<TResult> Default for AgentResultStreamTransformer<TResult> {
    fn default() -> Self {
        Self {
            result: PhantomData,
        }
    }
}

#[async_trait]
impl<TResult> TransformsOutgoingMessage for AgentResultStreamTransformer<TResult>
where
    TResult: TryFrom<OutgoingResponse, Error = OutgoingResponse> + Send + Sync + 'static,
{
    type Output = AgentResultStreamEvent<TResult>;

    async fn transform(
        &self,
        message: OutgoingMessage,
    ) -> Result<Vec<AgentResultStreamEvent<TResult>>, AgentRelayError> {
        match message {
            OutgoingMessage::Error(ErrorEnvelope { error, .. }) => {
                Ok(vec![AgentResultStreamEvent::WireError(error)])
            }
            notification @ OutgoingMessage::Notification(_) => {
                Err(AgentRelayError::MessageNotRelayable {
                    message: Box::new(notification),
                })
            }
            OutgoingMessage::Response(ResponseEnvelope {
                generated_by,
                request_id,
                response,
            }) => match TResult::try_from(response) {
                Ok(result) => Ok(vec![AgentResultStreamEvent::Result { request_id, result }]),
                Err(response) => Err(AgentRelayError::MessageNotRelayable {
                    message: Box::new(OutgoingMessage::Response(ResponseEnvelope {
                        generated_by,
                        request_id,
                        response,
                    })),
                }),
            },
        }
    }
}
