use serde::Deserialize;
use serde::Serialize;

use crate::generated_token_result::GeneratedTokenResult;

#[derive(Deserialize, Serialize)]
pub enum Response {
    GeneratedToken(GeneratedTokenResult),
    Timeout,
    TooManyBufferedRequests,
}
