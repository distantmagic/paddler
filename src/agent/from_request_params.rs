use tokio::sync::mpsc;

use crate::generated_token_envelope::GeneratedTokenEnvelope;

pub trait FromRequestParams: Send + Sync {
    type RequestParams;

    fn from_request_params(
        request_params: Self::RequestParams,
        generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
        generated_tokens_tx: mpsc::UnboundedSender<GeneratedTokenEnvelope>,
    ) -> Self;
}
