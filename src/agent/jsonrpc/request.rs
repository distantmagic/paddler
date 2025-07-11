use serde::Deserialize;
use serde::Serialize;

use super::request_params::GenerateTokens;

#[derive(Deserialize, Serialize)]
pub enum Request {
    GenerateTokens(GenerateTokens),
}
