use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::agent::from_request_params::FromRequestParams;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::request_params::ContinueFromConversationHistoryParams;

#[derive(Debug, Message)]
#[rtype(result = "Result<()>")]
pub struct ContinueFromConversationHistoryRequest {
    pub generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
    pub generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
    pub params: ContinueFromConversationHistoryParams,
}

impl FromRequestParams for ContinueFromConversationHistoryRequest {
    type RequestParams = ContinueFromConversationHistoryParams;

    fn from_request_params(
        params: Self::RequestParams,
        generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
        generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
    ) -> Self {
        ContinueFromConversationHistoryRequest {
            generate_tokens_stop_rx,
            generated_tokens_tx,
            params,
        }
    }
}
