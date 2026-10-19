use std::fmt::Debug;

use tokio_util::sync::CancellationToken;

use paddler_messaging::inference_client::message::Message as OutgoingMessage;
use paddler_messaging::inference_client::response::Response as OutgoingResponse;
use paddler_messaging::streamable_result::StreamableResult;

use crate::agent_streaming_request::AgentStreamingRequest;
use crate::controls_session::ControlsSession;
use crate::dispatched_agent::DispatchedAgent;
use crate::forward_responses_stream::forward_responses_stream;
use crate::inference_service::configuration::Configuration as InferenceServiceConfiguration;

pub async fn request_from_dispatched_agent<TControlsSession, TParams>(
    dispatched_agent: DispatchedAgent,
    connection_close: CancellationToken,
    inference_service_configuration: InferenceServiceConfiguration,
    params: TParams,
    request_id: String,
    session_controller: TControlsSession,
    shutdown: CancellationToken,
) where
    TControlsSession: ControlsSession<OutgoingMessage>,
    TParams: AgentStreamingRequest + Debug + Send,
    TParams::Response: Debug + Into<OutgoingResponse> + StreamableResult,
{
    let receive_response_controller = dispatched_agent
        .agent_controller
        .stream_responses_to(params, dispatched_agent.reserved_slots());

    forward_responses_stream(
        connection_close,
        dispatched_agent,
        inference_service_configuration,
        receive_response_controller,
        request_id,
        session_controller,
        shutdown,
    )
    .await;
}
