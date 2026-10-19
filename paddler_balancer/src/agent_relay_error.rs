use serde_json::Error as SerdeJsonError;
use thiserror::Error;

use paddler_messaging::inference_client::message::Message as OutgoingMessage;

#[derive(Debug, Error)]
pub enum AgentRelayError {
    #[error("Unable to serialize a message relayed to the client")]
    MessageUnserializable(#[source] SerdeJsonError),
    #[error("The client's protocol cannot carry this message: {message:?}")]
    MessageNotRelayable { message: Box<OutgoingMessage> },
}
