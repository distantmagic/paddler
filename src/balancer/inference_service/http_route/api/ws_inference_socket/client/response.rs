use serde::Deserialize;
use serde::Serialize;

use crate::generated_token_result::GeneratedTokenResult;

#[derive(Deserialize, Serialize)]
pub enum Response {
    GeneratedToken(GeneratedTokenResult),
    Timeout,
    TooManyBufferedRequests,
}

impl From<GeneratedTokenResult> for Response {
    fn from(result: GeneratedTokenResult) -> Self {
        Response::GeneratedToken(result)
    }
}
