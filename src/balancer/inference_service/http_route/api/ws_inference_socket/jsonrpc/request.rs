use serde::Deserialize;
use serde::Serialize;

use super::request_params::GenerateTokensParams;

#[derive(Deserialize, Serialize)]
pub enum Request {
    GenerateTokens(GenerateTokensParams),
}
