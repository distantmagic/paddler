use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::agent::from_request_params::FromRequestParams;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::request_params::GenerateTokensParams;

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateTokensRequest {
    pub generate_tokens_params: GenerateTokensParams,
    pub generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
    pub generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
}

impl FromRequestParams for GenerateTokensRequest {
    type RequestParams = GenerateTokensParams;

    fn from_request_params(
        request_params: Self::RequestParams,
        generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
        generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
    ) -> Self {
        GenerateTokensRequest {
            generate_tokens_params: request_params,
            generate_tokens_stop_rx,
            generated_tokens_tx,
        }
    }
}
