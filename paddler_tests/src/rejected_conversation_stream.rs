use tokio_util::sync::CancellationToken;

use paddler_client::client_inference::ClientInference;
use paddler_client::error::Result;
use paddler_client::inference_message_stream::InferenceMessageStream;

use crate::conversation_with_a_tool_requiring_an_undeclared_property::conversation_with_a_tool_requiring_an_undeclared_property;

pub async fn rejected_conversation_stream(
    client_inference: &ClientInference,
) -> Result<InferenceMessageStream> {
    client_inference
        .continue_from_conversation_history(
            CancellationToken::new(),
            conversation_with_a_tool_requiring_an_undeclared_property(),
        )
        .await
}
