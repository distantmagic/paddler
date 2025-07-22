use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::generated_token::GeneratedToken;
use crate::request_params::GenerateTokensParams;

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateTokensRequest {
    pub generate_tokens_params: GenerateTokensParams,
    pub generate_tokens_stop_rx: mpsc::UnboundedReceiver<()>,
    pub generated_tokens_tx: mpsc::UnboundedSender<GeneratedToken>,
    pub request_id: String,
}
