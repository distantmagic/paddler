use serde::Deserialize;
use serde::Serialize;

use crate::request_params::GenerateTokensParams;

#[derive(Deserialize, Serialize)]
pub enum Request {
    GenerateTokens(GenerateTokensParams),
}
