use serde::Deserialize;
use serde::Serialize;

use crate::generated_token::GeneratedToken;

#[derive(Debug, Deserialize, Serialize)]
pub enum GeneratedTokenResult {
    Done,
    Token(GeneratedToken),
}
