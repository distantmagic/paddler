use serde::Deserialize;
use serde::Serialize;

use super::Request;
use crate::jsonrpc::Error;
use crate::jsonrpc::RequestEnvelope;

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum Message {
    Error(Error),
    Request(RequestEnvelope<Request>),
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;
    use crate::request_params::GenerateTokensParams;

    #[test]
    fn test_message_serialization() -> Result<()> {
        let serialized = serde_json::to_string(&Message::Request(RequestEnvelope {
            id: "1".to_string(),
            request: Request::GenerateTokens(GenerateTokensParams {
                max_tokens: 500,
                prompt: "Hello, world!".to_string(),
            }),
        }))?;

        assert_eq!(
            serialized,
            r#"{"id":"1","request":{"GenerateTokens":{"prompt":"Hello, world!"}}}"#
        );

        Ok(())
    }
}
