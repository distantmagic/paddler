use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::agent::from_request_params::FromRequestParams;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::request_params::ContinueFromRawPromptParams;

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct ContinueFromRawPromptRequest {
    pub generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
    pub generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
    pub params: ContinueFromRawPromptParams,
}

impl FromRequestParams for ContinueFromRawPromptRequest {
    type RequestParams = ContinueFromRawPromptParams;

    fn from_request_params(
        params: Self::RequestParams,
        generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
        generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
    ) -> Self {
        ContinueFromRawPromptRequest {
            generate_tokens_stop_rx,
            generated_tokens_tx,
            params,
        }
    }
}
