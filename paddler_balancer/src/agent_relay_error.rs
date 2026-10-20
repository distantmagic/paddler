use serde_json::Error as SerdeJsonError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentRelayError {
    #[error("Unable to serialize a message relayed to the client")]
    MessageUnserializable(#[source] SerdeJsonError),
}
