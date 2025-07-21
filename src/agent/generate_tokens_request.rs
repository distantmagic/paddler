use actix::Message;
use anyhow::Result;
use tokio::sync::mpsc;

use crate::agent::generated_token::GeneratedToken;
use crate::request_params::GenerateTokensParams;

#[derive(Message)]
#[rtype(result = "Result<()>")]
pub struct GenerateTokensRequest {
    pub generated_tokens_tx: mpsc::UnboundedSender<GeneratedToken>,
    pub generate_tokens_params: GenerateTokensParams,
    pub request_id: String,
}
